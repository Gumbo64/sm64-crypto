# Happens to use debian thank goodness
FROM rust:latest
# Set the working directory in the container
WORKDIR /usr/src/sm64-crypto

# Copy the current directory contents into the container at WORKDIR
COPY . .

# Install any needed packages specified in requirements.txt
RUN apt-get update
RUN apt-get install -y make git build-essential pkg-config libusb-1.0-0-dev libsdl2-dev bsdmainutils

VOLUME ["/usr/src/sm64-crypto/prod"]

RUN make
# cleaning makes the image WAY smaller like 15GB vs 4GB
RUN cd rust_crypto && cargo clean && cd ../
CMD ["/bin/sh", "-c", "cd prod && ./main"]

