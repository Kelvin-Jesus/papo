class Papo < Formula
  desc "Direct P2P line between the Claude Code agents of different people"
  homepage "https://kelvin-jesus.github.io/papo/"
  version "0.4.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Kelvin-Jesus/papo/releases/download/v0.4.0/papo-v0.4.0-aarch64-apple-darwin.tar.gz"
      sha256 "976511f93083e1e54ef69f9a666be45fbc5b066f563e04cdb534d0200006686d"
    end
    on_intel do
      url "https://github.com/Kelvin-Jesus/papo/releases/download/v0.4.0/papo-v0.4.0-x86_64-apple-darwin.tar.gz"
      sha256 "09bf87eec68ada11a83efa5b2f9504a315723042748606ad57fbbfe882b2b976"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/Kelvin-Jesus/papo/releases/download/v0.4.0/papo-v0.4.0-aarch64-unknown-linux-musl.tar.gz"
      sha256 "ba2bbf951147f2fe78a3a097182e8e5fa899257bf13eede58ee85bfae96f3d19"
    end
    on_intel do
      url "https://github.com/Kelvin-Jesus/papo/releases/download/v0.4.0/papo-v0.4.0-x86_64-unknown-linux-musl.tar.gz"
      sha256 "8c7bd9cd2de99487df6e11001f1128df93bddce5bb9a29a532d1da046754c867"
    end
  end

  def install
    bin.install "papo"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/papo --version")
  end
end
