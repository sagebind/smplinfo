# smplinfo

A simple command line tool for reading and writing certain metadata in WAV files that are often used by hardware samplers. For example, this can be used to set the root note of a WAV file so that a sampler can know how to play the sample polyphonically.

## Installation
```
git clone https://github.com/sagebind/smplinfo.git
```

Navigate to the directory using 
```
cd smplinfo
```

Compiling CLI version of the tool:
```
cargo build --release
```

Compiling GUI version of the tool: <br/>

To compile the GUI version of the tool you first need to navigate to gui directory using
```
cd gui
```
And run
```
cargo build --release
```

The compiled binaries can be found in target directory of the project
```
cd ../target/release
```

## Running
To run CLI version
```
./smplinfo
```
To run GUI version
```
./smplinfo-gui
```

## License

This project's source code and documentation is licensed under the MIT license. See the [LICENSE](LICENSE) file for details.
