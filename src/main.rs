use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use iroh::{EndpointId, SecretKey};
use papo::{
    mcp::{self, Identity, Started, format_log_entry},
    net::bind_endpoint,
    node::{Node, NodeOptions, SendOutcome},
    proto::{PeerKind, now_ms, validate_name},
    room::{Invite, RoomSecret},
    store::{KnownPeer, Profile, Store},
};

/// Linha direta P2P entre os agentes de IA (Claude Code) de pessoas diferentes.
///
/// Fluxo típico: uma pessoa roda `papo new`, manda o convite para a outra, que roda
/// `papo join`. Cada uma roda `papo install` dentro do projeto e abre o Claude Code
/// com `claude --dangerously-load-development-channels server:papo`.
#[derive(Parser)]
#[command(name = "papo", version)]
struct Cli {
    /// Perfil a usar. Cada perfil é uma identidade numa sala; use vários para estar em
    /// várias salas.
    #[arg(long, global = true, env = "PAPO_PROFILE", default_value = "default")]
    profile: String,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Cria uma sala nova e mostra o convite para mandar ao colega.
    New {
        /// Seu nome na sala (ex.: kj). Os outros agentes vão te chamar assim.
        #[arg(long)]
        name: String,
        /// No que você está trabalhando (padrão: nome da pasta onde o Claude roda).
        #[arg(long)]
        about: Option<String>,
        /// Substitui um perfil existente.
        #[arg(long)]
        force: bool,
    },
    /// Entra numa sala usando o convite recebido.
    Join {
        /// O código de convite (começa com "papo1").
        invite: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        about: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// Mostra um convite para chamar mais alguém para a sua sala.
    Invite,
    /// Registra o papo no Claude Code (`claude mcp add`) para o projeto atual.
    Install {
        /// local = só este projeto (recomendado); user = todos os projetos; project = .mcp.json
        #[arg(long, value_enum, default_value_t = Scope::Local)]
        scope: Scope,
        /// Só imprime a configuração, sem executar `claude mcp add`.
        #[arg(long)]
        print: bool,
    },
    /// Servidor MCP via stdio. Quem executa isto é o Claude Code.
    Mcp,
    /// Manda uma mensagem sua (humano) para a sala.
    Say {
        /// Nome do destinatário (padrão: todos).
        #[arg(long)]
        to: Option<String>,
        /// O texto da mensagem.
        #[arg(required = true, trailing_var_arg = true)]
        text: Vec<String>,
    },
    /// Mostra a conversa da sala. Use -f para acompanhar ao vivo.
    Log {
        /// Quantas entradas mostrar.
        #[arg(short = 'n', long, default_value_t = 30)]
        lines: usize,
        /// Continua mostrando novas mensagens.
        #[arg(short, long)]
        follow: bool,
    },
    /// Testa a conexão: entra na sala e lista quem está online.
    Status {
        /// Quantos segundos esperar por alguém.
        #[arg(long, default_value_t = 20)]
        timeout: u64,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Scope {
    Local,
    User,
    Project,
}

impl Scope {
    fn as_str(self) -> &'static str {
        match self {
            Scope::Local => "local",
            Scope::User => "user",
            Scope::Project => "project",
        }
    }
}

const LAUNCH: &str = "claude --dangerously-load-development-channels server:papo";

#[tokio::main]
async fn main() {
    // Diagnostics go to stderr: stdout belongs to the MCP JSON-RPC stream.
    if let Ok(filter) = std::env::var("PAPO_LOG") {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
            .with_writer(std::io::stderr)
            .init();
    }
    let cli = Cli::parse();
    if let Err(e) = run(cli).await {
        eprintln!("erro: {e:#}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    let profile = cli.profile;
    match cli.cmd {
        Cmd::New { name, about, force } => cmd_new(&profile, name, about, force),
        Cmd::Join { invite, name, about, force } => cmd_join(&profile, &invite, name, about, force),
        Cmd::Invite => cmd_invite(&profile),
        Cmd::Install { scope, print } => cmd_install(&profile, scope, print),
        Cmd::Mcp => cmd_mcp(profile).await,
        Cmd::Say { to, text } => cmd_say(&profile, to, text.join(" ")).await,
        Cmd::Log { lines, follow } => cmd_log(&profile, lines, follow).await,
        Cmd::Status { timeout } => cmd_status(&profile, timeout).await,
    }
}

fn cmd_new(profile: &str, name: String, about: Option<String>, force: bool) -> Result<()> {
    validate_name(&name)?;
    let secret = RoomSecret::generate();
    let data = Profile { name: name.clone(), about, room: secret.to_base32(), created_ms: now_ms() };
    let store = Store::create(profile, &data, force)?;
    let me = store.secret_key()?.public();
    let invite = Invite { secret: secret.clone(), peers: vec![me] };
    println!("Sala {} criada. Você é \"{name}\" (perfil {profile}).\n", secret.room_id());
    println!("Mande este convite ao seu colega por um canal privado (quem tem o código entra na sala):\n");
    println!("  {}\n", invite.encode());
    println!("Ele roda:  papo join <convite> --name <nome-dele>");
    print_next_steps(profile);
    Ok(())
}

fn cmd_join(profile: &str, code: &str, name: String, about: Option<String>, force: bool) -> Result<()> {
    validate_name(&name)?;
    let invite = Invite::decode(code)?;
    let data = Profile { name: name.clone(), about, room: invite.secret.to_base32(), created_ms: now_ms() };
    let store = Store::create(profile, &data, force)?;
    let me = store.secret_key()?.public();
    let known = invite.peers.iter().filter(|id| **id != me).map(|id| (*id, KnownPeer::default())).collect();
    store.save_known_peers(&known)?;
    println!("Você entrou na sala {} como \"{name}\" (perfil {profile}).", invite.secret.room_id());
    print_next_steps(profile);
    Ok(())
}

fn print_next_steps(profile: &str) {
    let flag = if profile == "default" { String::new() } else { format!(" --profile {profile}") };
    println!("\nPróximos passos:");
    println!("  1. Dentro da pasta do projeto:  papo install{flag}");
    println!("  2. Abra o Claude Code ali:      {LAUNCH}");
    println!("  3. Peça algo como: \"combina com o agente do colega o formato da API pelo papo\"");
    println!("\nPara ver a conversa dos agentes ao vivo:  papo log -f{flag}");
}

fn cmd_invite(profile: &str) -> Result<()> {
    let store = Store::open(profile)?;
    let data = store.profile()?;
    let me = store.secret_key()?.public();
    let peers = invite_peers(me, store.known_peers()?);
    let invite = Invite { secret: data.room_secret()?, peers };
    println!("{}", invite.encode());
    Ok(())
}

/// Us first, then up to three members ordered by how recently we saw them, so the
/// newcomer can get in even when we are offline.
fn invite_peers(me: EndpointId, known: BTreeMap<EndpointId, KnownPeer>) -> Vec<EndpointId> {
    let mut others: Vec<(EndpointId, KnownPeer)> = known.into_iter().filter(|(id, _)| *id != me).collect();
    others.sort_by_key(|(_, p)| std::cmp::Reverse(p.last_seen_ms));
    std::iter::once(me).chain(others.into_iter().map(|(id, _)| id).take(3)).collect()
}

/// Arguments Claude Code passes to `papo`; the default profile stays implicit so the
/// generated config reads cleanly.
fn mcp_server_args(profile: &str) -> Vec<String> {
    let mut args = vec!["mcp".to_string()];
    if profile != "default" {
        args.extend(["--profile".to_string(), profile.to_string()]);
    }
    args
}

fn install_snippet(exe: &str, server_args: &[String]) -> serde_json::Value {
    serde_json::json!({"mcpServers": {"papo": {"command": exe, "args": server_args}}})
}

fn claude_add_args(scope: Scope, exe: &str, server_args: &[String]) -> Vec<String> {
    let mut cmd_args = vec![
        "mcp".to_string(),
        "add".into(),
        "--scope".into(),
        scope.as_str().into(),
        "papo".into(),
        // Everything after `--` belongs to our command, so `--profile` is not parsed
        // as a `claude` option.
        "--".into(),
        exe.to_string(),
    ];
    cmd_args.extend(server_args.iter().cloned());
    cmd_args
}

fn cmd_install(profile: &str, scope: Scope, print: bool) -> Result<()> {
    Store::open(profile)?; // fail early with setup instructions if unconfigured
    let exe = std::env::current_exe().context("locate the papo executable")?;
    let exe = exe.to_string_lossy().into_owned();
    let args = mcp_server_args(profile);

    if print {
        println!("{}", serde_json::to_string_pretty(&install_snippet(&exe, &args))?);
        return Ok(());
    }

    let status = run_claude(&claude_add_args(scope, &exe, &args));
    match status {
        Ok(s) if s.success() => {
            println!("\nPronto. Abra o Claude Code nesta pasta com:\n\n  {LAUNCH}\n");
            println!("Sem a flag também funciona, mas aí o Claude só vê mensagens novas quando chamar");
            println!("as ferramentas `wait`/`inbox` do papo (ex.: \"espera a resposta do colega\").");
            Ok(())
        }
        Ok(s) => bail!(
            "`claude mcp add` falhou ({s}). Se já existe um servidor \"papo\", remova com \
             `claude mcp remove papo --scope {}` e rode de novo.",
            scope.as_str()
        ),
        Err(_) => {
            println!("Não achei o comando `claude` no PATH. Rode manualmente:\n");
            println!("  claude mcp add --scope {} papo -- \"{exe}\" {}", scope.as_str(), args.join(" "));
            Ok(())
        }
    }
}

/// `claude` is a .exe from the native installer but a .cmd shim when installed via npm
/// on Windows; `Command` does not resolve .cmd on its own.
fn run_claude(args: &[String]) -> std::io::Result<std::process::ExitStatus> {
    match Command::new("claude").args(args).status() {
        Err(e) if cfg!(windows) && e.kind() == std::io::ErrorKind::NotFound => {
            Command::new("claude.cmd").args(args).status()
        }
        other => other,
    }
}

async fn cmd_mcp(profile: String) -> Result<()> {
    let identity = Store::open(&profile).ok().and_then(|s| s.profile().ok()).and_then(|p| {
        let room_id = p.room_secret().ok()?.room_id();
        Some(Identity { name: p.name, room_id })
    });
    mcp::serve(identity, start_agent(profile)).await
}

async fn start_agent(profile: String) -> Result<Started> {
    let store = Store::open(&profile)?;
    let lock = store.lock()?;
    let data = store.profile()?;
    let room = data.room_secret()?;
    // Claude Code starts MCP servers in the project directory, which tells peers what
    // this agent is working on.
    let about = data.about.clone().or_else(|| std::env::current_dir().ok().and_then(|d| about_from_dir(&d)));
    let endpoint = bind_endpoint(store.secret_key()?).await?;
    let opts =
        NodeOptions { name: data.name.clone(), about, kind: PeerKind::Agent, ephemeral: false, bootstrap: vec![] };
    let node = Node::spawn(endpoint, room, opts, Some(store.clone())).await?;
    Ok(Started { node, store, profile: data, lock })
}

/// The project folder name is a good default for "what I'm working on".
fn about_from_dir(dir: &Path) -> Option<String> {
    dir.file_name().map(|n| n.to_string_lossy().into_owned()).filter(|s| !s.is_empty())
}

/// One-shot node with a throwaway identity, so it can run next to the MCP server that
/// owns the profile's real identity.
async fn ephemeral_node(store: &Store, data: &Profile) -> Result<Node> {
    let bootstrap: Vec<_> = store.known_peers()?.into_keys().collect();
    if bootstrap.is_empty() {
        bail!("ainda não conheço ninguém nesta sala. Mande o convite (`papo invite`) e espere o colega entrar.");
    }
    let endpoint = bind_endpoint(SecretKey::generate()).await?;
    let opts = NodeOptions { name: data.name.clone(), about: None, kind: PeerKind::Human, ephemeral: true, bootstrap };
    Node::spawn(endpoint, data.room_secret()?, opts, None).await
}

async fn cmd_say(profile: &str, to: Option<String>, text: String) -> Result<()> {
    let store = Store::open(profile)?;
    let data = store.profile()?;
    let node = ephemeral_node(&store, &data).await?;
    eprintln!("conectando…");
    if !node.wait_for_neighbor(Duration::from_secs(30)).await {
        node.shutdown().await;
        bail!("ninguém da sala está online agora; a mensagem não foi enviada");
    }
    let outcome = node.send(&text, to, None, Duration::from_secs(15)).await?;
    match outcome {
        SendOutcome::Delivered { id, by } => println!("entregue a {by} (msg {id})"),
        SendOutcome::Queued { id } => println!("enviada (msg {id}), mas ninguém confirmou o recebimento ainda"),
    }
    node.shutdown().await;
    Ok(())
}

async fn cmd_log(profile: &str, lines: usize, follow: bool) -> Result<()> {
    let store = Store::open(profile)?;
    let me = store.profile()?.name;
    let entries = store.read_log()?;
    for entry in &entries[entries.len().saturating_sub(lines)..] {
        println!("{}", format_log_entry(entry, &me));
    }
    if !follow {
        return Ok(());
    }
    follow_log(store.log_path(), &me).await
}

/// Polls instead of using filesystem notifications: works the same on every OS and
/// the log is tiny.
async fn follow_log(path: PathBuf, me: &str) -> Result<()> {
    let mut offset = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let Ok(mut file) = std::fs::File::open(&path) else { continue };
        let len = file.metadata()?.len();
        if len < offset {
            offset = 0; // log was truncated or replaced
        }
        if len == offset {
            continue;
        }
        file.seek(SeekFrom::Start(offset))?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            if !line.ends_with('\n') {
                break; // partial write; pick it up on the next poll
            }
            offset += line.len() as u64;
            if let Ok(entry) = serde_json::from_str::<papo::store::LogEntry>(&line) {
                println!("{}", format_log_entry(&entry, me));
            }
            line.clear();
        }
    }
}

async fn cmd_status(profile: &str, timeout: u64) -> Result<()> {
    let store = Store::open(profile)?;
    let data = store.profile()?;
    let room = data.room_secret()?;
    println!("Perfil {profile}: você é \"{}\" na sala {}.", data.name, room.room_id());
    println!("Identidade do agente: {}", store.secret_key()?.public());
    let node = ephemeral_node(&store, &data).await?;
    println!("Procurando membros da sala (até {timeout}s)…");
    let connected = node.wait_for_neighbor(Duration::from_secs(timeout)).await;
    if connected {
        // Give presence announcements a moment to arrive after the link comes up.
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
    let peers = node.peers();
    for p in peers.iter().filter(|p| !p.ephemeral) {
        let name = p.name.clone().unwrap_or_else(|| "(nome desconhecido)".into());
        let state = if p.online { "online" } else { "offline" };
        let about = p.about.as_ref().map(|a| format!(" — {a}")).unwrap_or_default();
        println!("  {name} [{}] {state}{about}", p.node.fmt_short());
    }
    if !connected {
        println!("Ninguém respondeu. O colega está com o Claude Code aberto (papo rodando)?");
    }
    let pending = store.outbox()?.len();
    let unread = store.inbox()?.len();
    println!("Mensagens na fila de envio: {pending}. Não lidas pelo agente: {unread}.");
    node.shutdown().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(seed: u8) -> EndpointId {
        SecretKey::from_bytes(&[seed; 32]).public()
    }

    #[test]
    fn invite_lists_me_first_then_the_three_most_recently_seen_members() {
        let me = id(1);
        let known: BTreeMap<_, _> = [(id(2), 10), (id(3), 40), (id(4), 30), (id(5), 20), (me, 99)]
            .into_iter()
            .map(|(peer, seen)| (peer, KnownPeer { name: None, last_seen_ms: seen }))
            .collect();
        assert_eq!(invite_peers(me, known), vec![me, id(3), id(4), id(5)]);
    }

    #[test]
    fn invite_with_no_known_members_is_just_me() {
        assert_eq!(invite_peers(id(1), BTreeMap::new()), vec![id(1)]);
    }

    #[test]
    fn default_profile_stays_implicit_in_server_args() {
        assert_eq!(mcp_server_args("default"), vec!["mcp"]);
        assert_eq!(mcp_server_args("time-b"), vec!["mcp", "--profile", "time-b"]);
    }

    #[test]
    fn claude_add_separates_our_flags_after_double_dash() {
        let args = claude_add_args(Scope::Local, "/bin/papo", &mcp_server_args("x"));
        assert_eq!(args, ["mcp", "add", "--scope", "local", "papo", "--", "/bin/papo", "mcp", "--profile", "x"]);
        assert_eq!(claude_add_args(Scope::User, "p", &[])[3], "user");
        assert_eq!(claude_add_args(Scope::Project, "p", &[])[3], "project");
    }

    #[test]
    fn install_snippet_is_a_valid_mcp_json_entry() {
        let snippet = install_snippet("/opt/papo", &mcp_server_args("default"));
        assert_eq!(snippet["mcpServers"]["papo"]["command"], "/opt/papo");
        assert_eq!(snippet["mcpServers"]["papo"]["args"], serde_json::json!(["mcp"]));
    }

    #[test]
    fn about_defaults_to_the_folder_name() {
        assert_eq!(about_from_dir(Path::new("/home/ana/api-pagamentos")).as_deref(), Some("api-pagamentos"));
        assert_eq!(about_from_dir(Path::new("/")), None);
    }

    #[test]
    fn cli_definition_is_consistent() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
