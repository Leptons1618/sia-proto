#!/bin/bash
set -e

# Script to test SIA AppImage using Docker

echo "🐳 Testing SIA AppImage with Docker..."
echo ""

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker first."
    exit 1
fi

# Build and test the AppImage
echo "📦 Building Docker image and testing AppImage..."
docker build -f Dockerfile.test-appimage -t sia-appimage-test:latest .

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ AppImage test completed successfully!"
    echo ""
    echo "To inspect the AppImage:"
    echo "  docker run -it --rm -v \$(pwd)/appimage-build:/build/appimage-build sia-appimage-test:latest"
    echo ""
    echo "To run the extracted AppImage:"
    echo "  docker run -it --rm -v \$(pwd)/appimage-build:/build/appimage-build sia-appimage-test:latest bash"
    echo "  # Then inside container:"
    echo "  cd /build/appimage-build/squashfs-root"
    echo "  ./AppRun --help"
else
    echo ""
    echo "❌ AppImage test failed. Check the output above for errors."
    exit 1
fi

