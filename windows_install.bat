wsl --install --no-distribution
wsl --install -d Debian
wsl --cd "%~dp0" -d Debian -e bash -c "cat install.sh | tr -d '\r' | bash"