class BrewMaintainer < Formula
  desc "Automated Homebrew maintenance tool (update, upgrade, cleanup with logs)"
  homepage "https://github.com/<REPLACED_BY_GITHUB_ACTION>"
  version "<REPLACED_BY_GITHUB_ACTION>"
  url "<REPLACED_BY_GITHUB_ACTION>"
  sha256 "<REPLACED_BY_GITHUB_ACTION>"
  license "MIT"

  depends_on :macos # only macOS supported

  def install
    bin.install "brew-maintainer"
  end

  service do
    run [opt_bin/"brew-maintainer"]
    run_at_load true
    keep_alive false
    interval 21600  # 6 hours = 21600 seconds
    log_path var/"log/brew-maintainer.log"
    error_log_path var/"log/brew-maintainer.err.log"
    working_dir var
    environment_variables PATH: "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"
  end

  def plist
      <<~PLIST
        <?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
        <plist version="1.0">
          <dict>
            <key>Label</key>
            <string>#{plist_name}</string>
            <key>ProgramArguments</key>
            <array>
              <string>#{opt_bin}/brew-maintainer</string>
            </array>
            <key>StartInterval</key>
            <integer>21600</integer>
            <key>RunAtLoad</key>
            <true/>
            <key>WorkingDirectory</key>
            <string>#{var}</string>
            <key>StandardOutPath</key>
            <string>#{var}/log/brew-maintainer.log</string>
            <key>StandardErrorPath</key>
            <string>#{var}/log/brew-maintainer.err.log</string>
            <key>EnvironmentVariables</key>
            <dict>
              <key>PATH</key>
              <string>/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin</string>
            </dict>
          </dict>
        </plist>
        PLIST
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
