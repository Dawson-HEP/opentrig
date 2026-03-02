# Opentrig CLI

## Install udev rules

- `mv ./99-opentrig.rules /etc/udev/rules.d/99-opentrig.rules`
- `sudo usermod -a -G users username`
- `sudo udevadm control --reload`
- `sudo udevadm trigger`

