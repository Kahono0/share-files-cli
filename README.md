# sharef
Send files to users on your network using a simple command line interface.

## Installation
Download the release of your os [here](https://github.com/Kahono0/share-files-cli/releases/tag/v0.0.1)

## Building
On unix systems
```bash
chmod +x sharef
./sharef
```
On windows
```bash
sharef.exe
```

## Usage
### Sending files
```bash
sharef s <filename>
```

### Sending folders
```bash
sharef sf <foldername>
```

### Receiving files
```bash
sharef r <address>
```

### Receiving folders
```bash
sharef rf <address>
```

Note: The address will be given by the sender.

## known issues
- The sender and receiver must be on the same network.
- The receiver must be running the program before the sender sends the file.
- receiver cannot connect to some machines on the network (currently researching the issue).

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

