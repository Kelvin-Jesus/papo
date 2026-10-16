class Papo < Formula
  desc "Direct P2P line between the Claude Code agents of different people"
  homepage "https://kelvin-jesus.github.io/papo/"
  version "0.3.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Kelvin-Jesus/papo/releases/download/v0.3.0/papo-v0.3.0-aarch64-apple-darwin.tar.gz"
      sha256 "4d8983f0fc5e01ea3199a3094f5f1158cee1c6abafb4635b4b89e9eda7dff5fb"
    end
    on_intel do
      url "https://github.com/Kelvin-Jesus/papo/releases/download/v0.3.0/papo-v0.3.0-x86_64-apple-darwin.tar.gz"
      sha256 "f362d07887d0fdb7b006e0e4525346230c33f9df200da9fb9223cecd53c2848a"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/Kelvin-Jesus/papo/releases/download/v0.3.0/papo-v0.3.0-aarch64-unknown-linux-musl.tar.gz"
      sha256 "c11bcf79f306a89a1a844b77771687e0aff31ae5d984f27c6fc14bce41e9d880"
    end
    on_intel do
      url "https://github.com/Kelvin-Jesus/papo/releases/download/v0.3.0/papo-v0.3.0-x86_64-unknown-linux-musl.tar.gz"
      sha256 "c24466d38019cb4481929aca2ff3a4733ca1b99c3660ead6510af5ca92d87ccc"
    end
  end

  def install
    bin.install "papo"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/papo --version")
  end
end
