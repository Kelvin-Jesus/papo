class Papo < Formula
  desc "Direct P2P line between the Claude Code agents of different people"
  homepage "https://kelvin-jesus.github.io/papo/"
  version "0.3.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Kelvin-Jesus/papo/releases/download/v0.3.1/papo-v0.3.1-aarch64-apple-darwin.tar.gz"
      sha256 "ddd9daa5d89d316e81156936f3003e848823c121ba409c6eee962a0f1cf90e89"
    end
    on_intel do
      url "https://github.com/Kelvin-Jesus/papo/releases/download/v0.3.1/papo-v0.3.1-x86_64-apple-darwin.tar.gz"
      sha256 "a0e5502e08f067be72580c76dea07872b7a8e0c022a6c4be7de7bfb4ca838279"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/Kelvin-Jesus/papo/releases/download/v0.3.1/papo-v0.3.1-aarch64-unknown-linux-musl.tar.gz"
      sha256 "94a5595cf86dc69831ea1c4796d0f8ed9aa85676bc93986a8a2fb77b05a37aea"
    end
    on_intel do
      url "https://github.com/Kelvin-Jesus/papo/releases/download/v0.3.1/papo-v0.3.1-x86_64-unknown-linux-musl.tar.gz"
      sha256 "c55d6cbc9b3644e8628f02e62769d7e71b8819683373a1fcca81c33999405428"
    end
  end

  def install
    bin.install "papo"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/papo --version")
  end
end
