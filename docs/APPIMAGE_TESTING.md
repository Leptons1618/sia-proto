# AppImage Testing Guide

This guide explains how to test the SIA AppImage using Docker to ensure it builds correctly and is functional.

## Why Test AppImage in Docker?

Testing AppImages in Docker provides:
- **Clean environment**: Ensures the AppImage works in a fresh Linux system
- **Reproducible tests**: Same environment every time
- **Isolation**: Doesn't affect your host system
- **CI/CD ready**: Can be integrated into automated testing pipelines

## Quick Test

Run the automated test script:

```bash
./test-appimage.sh
```

This will:
1. Build the AppImage inside a Docker container
2. Extract and verify the AppImage contents
3. Test that all binaries and files are present
4. Verify binaries are functional

## Manual Testing

### Build and Test

```bash
# Build the Docker image
docker build -f Dockerfile.test-appimage -t sia-appimage-test:latest .

# Run the container interactively
docker run -it --rm sia-appimage-test:latest bash
```

### Inside the Container

```bash
# Check AppImage exists
ls -lh appimage-build/sia-x86_64.AppImage

# Extract AppImage
cd appimage-build
./sia-x86_64.AppImage --appimage-extract

# Test binaries
./squashfs-root/usr/bin/sia-agent --help
./squashfs-root/usr/bin/sia-cli --help

# Test CLI with AppRun
cd squashfs-root
./AppRun --help
```

### Test Full Functionality

To test the full functionality, you'll need to run the agent:

```bash
# Inside the extracted AppImage directory
cd squashfs-root

# Start the agent in background
./usr/bin/sia-agent &

# Wait a moment for it to start
sleep 2

# Test CLI
./usr/bin/sia-cli status
./usr/bin/sia-cli list

# Or use AppRun
./AppRun status
```

## Using Docker Compose

```bash
# Build and run
docker-compose -f docker-compose.test-appimage.yml up --build

# Access the container
docker exec -it sia-appimage-test bash
```

## What Gets Tested

The Dockerfile tests:

1. ✅ **AppImage Creation**: Verifies the AppImage file is created
2. ✅ **File Format**: Checks it's a valid AppImage
3. ✅ **Extraction**: Tests that the AppImage can be extracted
4. ✅ **Binaries**: Verifies `sia-agent` and `sia-cli` exist
5. ✅ **Executability**: Tests that binaries can run
6. ✅ **CLI Files**: Checks TypeScript CLI files are included
7. ✅ **Configuration**: Verifies config files are present

## Limitations

- **FUSE Requirement**: AppImages typically need FUSE to run directly. In Docker, we extract them instead.
- **System Service**: The agent can't run as a systemd service in Docker, but can run manually.
- **Socket Permissions**: Unix sockets work in Docker, but paths may differ.

## CI/CD Integration

You can integrate this into CI/CD:

```yaml
# Example GitHub Actions
- name: Test AppImage
  run: |
    docker build -f Dockerfile.test-appimage -t sia-appimage-test .
    docker run --rm sia-appimage-test:latest
```

## Troubleshooting

### AppImage won't extract

```bash
# Make sure it's executable
chmod +x appimage-build/sia-x86_64.AppImage

# Try extracting manually
./appimage-build/sia-x86_64.AppImage --appimage-extract
```

### Binaries not found

Check the AppImage structure:
```bash
cd appimage-build/squashfs-root
find . -name "sia-*" -type f
```

### Permission errors

The Dockerfile sets proper permissions, but if you see issues:
```bash
chmod +x appimage-build/squashfs-root/usr/bin/*
```

## Next Steps

After successful testing:
1. Test on a real Linux system (without Docker)
2. Test on different Linux distributions
3. Verify all features work correctly
4. Test installation and uninstallation
5. Performance testing

