# Happens to use debian thank goodness
FROM rust:latest
# Set the working directory in the container
WORKDIR /usr/src/sm64-crypto

# Copy the current directory contents into the container at WORKDIR
COPY . .

# Install any needed packages specified in requirements.txt
RUN apt-get update && apt-get install -y make
RUN apt-get install -y git build-essential pkg-config libusb-1.0-0-dev libsdl2-dev

VOLUME ["/usr/src/sm64-crypto/prod"]


EXPOSE 80
EXPOSE 8080 9090 3000

# add SM64_ARG="--mine" if you want to mine
ENV SM64_ARG=""
CMD ["/bin/sh", "-c", "if [ -d ./prod ] && [ -f ./prod/main ]; then cd ./prod && ./main $SM64_ARG; else cp ./prod/baserom.us.z64 . && make && cd ./prod && ./main $SM64_ARG; fi"]