class BrewMaintainer < Formula
  desc "Automated Homebrew maintenance tool (update, upgrade, cleanup with logs)"
  homepage "https://github.com/<REPLACED_BY_GITHUB_ACTION>"
  url "<REPLACED_BY_GITHUB_ACTION>"
  version "<REPLACED_BY_GITHUB_ACTION>"
  sha256 "<REPLACED_BY_GITHUB_ACTION>"
  license "MIT"

  depends_on :macos # only macOS supported

  def install
    bin.install "brew-maintainer"
  end

  service do
    run [opt_bin/"brew-maintainer"]
    run_type :interval
    interval 21600  # 6 hours = 21600 seconds
    log_path var/"log/brew-maintainer.log"
    error_log_path var/"log/brew-maintainer.err.log"
    working_dir var
    environment_variables(
      HOME => ENV[“HOME"],
      PATH => std_service_path_env
    )
  end

  def caveats
    <<~DESC
      brew-maintainer will automatically run every 6 hours via macOS service.
      Logs are stored under:
        #{var}/log/brew-maintainer.log
      If a run requires user input, it will be skipped and noted in the log.
    DESC
  end
end
