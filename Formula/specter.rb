class Specter < Formula
  desc "Rust-powered spec-driven development tool with AI orchestration"
  homepage "https://github.com/chrischeng-c4/specter"
  url "https://github.com/chrischeng-c4/specter/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"
  head "https://github.com/chrischeng-c4/specter.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "specter #{version}", shell_output("#{bin}/specter --version")
  end
end
