# Happens to use debian thank goodness
FROM rust:latest
# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the current directory contents into the container at /usr/src/app
COPY . .

# Define a build argument for the file
ARG SM64_ROM

# Install any needed packages specified in requirements.txt
RUN apt-get update && apt-get install -y make
RUN apt-get install -y git build-essential pkg-config libusb-1.0-0-dev libsdl2-dev



VOLUME ["/prod"]


EXPOSE 80
EXPOSE 8080 9090 3000

CMD ["bash", "-c", "if [ -f /prod/main ]; then cd prod && ./main; else cp /prod/baserom.us.z64 baserom.us.z64 && make && cd prod && ./main; fi"]
