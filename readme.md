

# SM64 Crypto

## Summary
<!-- Fill in a brief summary of your project here. Describe its purpose, features, and any other relevant information. -->
This project manages a blockchain (perhaps a cryptocurrency in future) where instead of mining, players play Mario 64 to create blocks. At the moment it has the sm64 mining/evaluating, the blockchain itself, networking, and consensus.

## Usage
To use the project, follow the installation, cd into `prod` and then run `./main` to see the commands


## Installation
Before you begin, ensure you have the following:

- **Rust**: Follow the instructions on the [official Rust website](https://www.rust-lang.org/tools/install) to install Rust and Cargo.
- **SM64 z64**: Ensure that you legally obtain a US copy of the game as a z64 file.

The following instructions are based off the [sm64-port repository](https://www.github.com/sm64-port/sm64-port)

### Windows (WSL) pre-steps
1. Run terminal as administrator and install WSL using `wsl --install`
2. Open debian in WSL `wsl -d Debian`

### Installation

1. Clone/download this repo and then cd into it.
<!-- 2. Clone the repo: `git clone https://github.com/sm64-port/sm64-port.git`, which will create a directory `sm64-port` and then **enter** it `cd sm64-port`. -->
2. Place the sm64 z64 file renamed to `baserom.us.z64` into the repository's root directory.
3. Run `./install.sh` to install requirements and build the project.
4. The executable binary will be located at `prod/main`


### Podman Desktop (no GUI or mining)
1. Put baserom.us.z64 in the root folder (next to readme.md etc)
2. On podman desktop, go Containers -> Create -> Containerfile or Dockerfile (purple)
3. Select Containerfile path by navigating to this folder, then selecting Dockerfile
4. Name it sm64-crypto optionally
5. Click Build and wait for it to build
6. Once it's done, go to Images and click the icon next to the sm64-crypto image that looks like a play button
7. go to the bottom and click Start Container

<!-- ### Windows

1. Install and update MSYS2, following all the directions listed on https://www.msys2.org/.
2. From the start menu, launch MSYS2 MinGW and install required packages depending on your machine (do **NOT** launch "MSYS2 MSYS"):
  * 64-bit: Launch "MSYS2 MinGW 64-bit" and install: `pacman -S git make python3 mingw-w64-x86_64-gcc`
  * 32-bit (will also work on 64-bit machines): Launch "MSYS2 MinGW 32-bit" and install: `pacman -S git make python3 mingw-w64-i686-gcc`
  * Do **NOT** by mistake install the package called simply `gcc`.
3. The MSYS2 terminal has a _current working directory_ that initially is `C:\msys64\home\<username>` (home directory). At the prompt, you will see the current working directory in yellow. `~` is an alias for the home directory. You can change the current working directory to `My Documents` by entering `cd /c/Users/<username>/Documents`.
4. Clone the repo: `git clone https://github.com/sm64-port/sm64-port.git`, which will create a directory `sm64-port` and then **enter** it `cd sm64-port`.
5. Place a *Super Mario 64* ROM called `baserom.<VERSION>.z64` into the repository's root directory for asset extraction, where `VERSION` can be `us`, `jp`, or `eu`.
6. Run `make` to build. Qualify the version through `make VERSION=<VERSION>`. Add `-j4` to improve build speed (hardware dependent based on the amount of CPU cores available).
7. The executable binary will be located at `build/<VERSION>_pc/sm64.<VERSION>.f3dex2e.exe` inside the repository. -->

## Credits
* Iroh
* sm64-port