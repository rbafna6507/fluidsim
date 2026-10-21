# Live IMU viewer

The Raspberry Pi sends pitch and roll to a small viewer running on your Mac. The Pi sends radians over UDP; the viewer shows degrees and rotates a simple 3D MPU board model.

## On the Mac

From the project directory, start the receiver and web page:

```sh
node viewer/server.js
```

Open <http://localhost:8080>. Keep this terminal running. The viewer listens for UDP packets on port `41234` and serves its page on port `8080`. Three.js is loaded from jsDelivr, so the browser needs internet access.

Find the Mac's address on the local network with:

```sh
ipconfig getifaddr en0
```

If that prints nothing, find the active interface address in macOS Network settings.

## On the Raspberry Pi

Run the existing program with the Mac's address and UDP port:

```sh
./fluid --udp 192.168.1.23:41234
```

Replace the example address with the Mac's address. Both devices need to be on a network that allows traffic between them. Without `--udp`, the program keeps its existing console-only behavior.
