# SM64 Crypto
## Summary
Creating an independent cryptocurrency gained exclusively by playing Mario 64. Since gameplay *directly* seals blocks, there is no need for the initial investments, PVP or gas fees that other play-to-earn cryptos require. Even compared to normal cryptocurrencies it has many unique properties, such as how it is only mined by humans (at the moment) and produces a usable dataset of gameplay as the blockchain grows.

Functional at the moment is the sm64 playing/verifying, the blockchain itself (without transactions/wallets), networking, and consensus.
To mine a block, you must obtain 1 star. So you need to go to the top of bobomb battlefield's mountain and defeat King Bobomb.

## Installation
Before you begin, ensure you have the following:
- **SM64 z64**: Ensure that you legally obtain a US copy of the game as a z64 file. It should be 8.00MB large

### Windows steps
Make sure that HyperV and Virtualisation are on so that WSL can work. You may have to restart your computer when WSL is installed.
You may have to install WSL manually if this install script fails.

1. download this repository as zip
2. extract it
3. Put your sm64 ROM into the root of the extracted folder, and call it `baserom.us.z64`
4. click and run `windows_install.bat`
5. do what it says (it will take a while). if you get coloured text after Debian installs then type `exit` and press enter to continue installation.
  When rust is installing, press enter if you don't have rust or press 3 if you already have rust
6. after it is done, click `windows_play_game.bat`

### Linux steps
1. download this repository as zip
2. extract it
3. Put your sm64 ROM into the root of the extracted folder, and call it `baserom.us.z64`
4. right click and run `install.sh` (or cd into the root of the repository and run it in the terminal)
5. do what it says (it will take a while)
  When rust is installing, press enter if you don't have rust or press 3 if you already have rust
6. after it is done, run `play_game.sh` to play

### Podman Desktop (no GUI or mining)
1. Put baserom.us.z64 in the root folder (next to readme.md etc)
2. On podman desktop, go Containers -> Create -> Containerfile or Dockerfile (purple)
3. Select Containerfile path by navigating to this folder, then selecting Dockerfile
4. Name it sm64-crypto optionally
5. Click Build and wait for it to build
6. Once it's done, go to Images and click the icon next to the sm64-crypto image that looks like a play button
7. go to the bottom and click Start Container

## Controls
Keyboard button - N64 Button
* WASD - Joystick
* I - A
* J - B
* O - Z
* Enter - Start

Special keys
* R - Go back ~4 seconds in the run
* Q - Completely restart run
* hold Space - Change to 0.1x Speed
* hold (Shift + Space) - Change to 10x Speed (or as fast as your computer can go)


## Credits
* Iroh
* sm64-port
