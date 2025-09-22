wsl --install Debian
wsl --cd "%~dp0" -d Debian -e bash -c "make"