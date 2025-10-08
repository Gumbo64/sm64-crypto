# SM64 Crypto
## Summary
Creating an independent cryptocurrency gained exclusively by playing Mario 64. 
Since gameplay *directly* seals blocks, it has many unique properties: 
* No need for the initial investments, PVP or gas fees that required for other play-to-earn cryptos require
* Only mined by humans at the moment, otherwise it could be an open-invite benchmark for AI
* Produces a usable dataset of gameplay as the blockchain grows.

To successfully finish your gameplay, you must obtain 1 star. Therefore you must to go to the top of bobomb battlefield's mountain and defeat King Bobomb.

Unique problems require unique solutions, so some mechanisms have been invented:
* While playing, your button inputs are randomly perturbed (like poking a robot) to prevent hard-coded or recycled gameplay.
* The random seed is calculated based on the details of the block, so the RNG is verifiable and the gameplay is linked to the block
* Instead of pre-computing all the random pertubations at the beginning, the RNG factors in Mario's current position and velocity after each frame of gameplay (unpredictable). This prevents players from abusing compute power to cherrypick hassle-free seeds.

Progress:
- [x] Mario 64 integration: Playing and verifying
- [x] Block creation, storage, chain
- [x] P2P Networking, broadcasting blocks
- [x] Consensus and syncing
- [x] Calculating random seeds and random input generation
- [ ] Wallets
- [ ] Transactions

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

## Executable options
Options:
  -m, --mine                     Enable mining
  -n, --nowait                   Wait for a connection before starting
  -s, --showblocks               
  -m, --miner-name <MINER_NAME>  [default: Gumbo64]
  -h, --help                     Print help

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
6. after it is done, click `windows_play_game.bat` OR cd into `prod` and run `./main` using your chosen commands

### Linux steps
1. download this repository as zip
2. extract it
3. Put your sm64 ROM into the root of the extracted folder, and call it `baserom.us.z64`
4. right click and run `install.sh` (or cd into the root of the repository and run it in the terminal)
5. do what it says (it will take a while)
  When rust is installing, press enter if you don't have rust or press 3 if you already have rust
6. after it is done, run `play_game.sh` to play OR cd into `prod` and run `./main` using your chosen commands

### Podman Desktop (no GUI or mining)
1. Put baserom.us.z64 in the root folder (next to readme.md etc)
2. On podman desktop, go Containers -> Create -> Containerfile or Dockerfile (purple)
3. Select Containerfile path by navigating to this folder, then selecting Dockerfile
4. Name it sm64-crypto optionally
5. Click Build and wait for it to build
6. Once it's done, go to Images and click the icon next to the sm64-crypto image that looks like a play button
7. go to the bottom and click Start Container

## Credits
* Iroh
* sm64-port
