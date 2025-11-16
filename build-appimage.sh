#!/bin/bash
set -e

# SIA AppImage Build Script
# Creates a portable AppImage package containing both agent and CLI

echo "📦 Building SIA AppImage..."

# Get project directory FIRST, before any PATH modifications
# Use absolute path to ensure it works even if PATH is modified
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$PROJECT_DIR/appimage-build"
APPIMAGE_DIR="$BUILD_DIR/sia.AppDir"

# Get the original user (who ran sudo) and their environment
ORIGINAL_USER="${SUDO_USER:-$USER}"
ORIGINAL_HOME=$(eval echo ~$ORIGINAL_USER)

# Preserve system PATH - always include essential system directories
SYSTEM_PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

# Get the original user's PATH when running with sudo (for npm/node)
if [ -n "$SUDO_USER" ]; then
    # Get the user's actual PATH by running a command as them
    USER_PATH=$(sudo -u "$ORIGINAL_USER" -i bash -c 'echo $PATH' 2>/dev/null || echo "")
    if [ -n "$USER_PATH" ]; then
        # Combine system PATH with user PATH, prioritizing system paths for basic commands
        export PATH="$SYSTEM_PATH:$USER_PATH"
    else
        # Fallback: try common locations, but keep system paths first
        export PATH="$SYSTEM_PATH:$ORIGINAL_HOME/.local/bin:$ORIGINAL_HOME/.cargo/bin"
        # Add nvm paths if they exist
        if [ -d "$ORIGINAL_HOME/.nvm" ]; then
            export PATH="$PATH:$ORIGINAL_HOME/.nvm/current/bin"
            for node_dir in "$ORIGINAL_HOME/.nvm/versions/node"/*/bin; do
                if [ -d "$node_dir" ]; then
                    export PATH="$PATH:$node_dir"
                fi
            done
        fi
    fi
else
    # Not using sudo, keep current PATH but ensure system paths are included
    export PATH="$SYSTEM_PATH:$PATH"
fi

# Clean previous build
rm -rf "$BUILD_DIR"
mkdir -p "$APPIMAGE_DIR"

# Check dependencies
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if Rust toolchain is configured
if ! rustup show default &> /dev/null 2>&1; then
    echo "⚠️  No default Rust toolchain set."
    if rustup toolchain list 2>/dev/null | grep -q "stable"; then
        echo "   Setting default to stable..."
        rustup default stable || {
            echo "❌ Failed to set default toolchain. Please run: rustup default stable"
            exit 1
        }
    else
        echo "   Installing stable toolchain..."
        rustup toolchain install stable || {
            echo "❌ Failed to install stable toolchain. Please check your internet connection."
            exit 1
        }
        rustup default stable || {
            echo "❌ Failed to set default toolchain. Please run: rustup default stable"
            exit 1
        }
    fi
fi

# Find node and npm - try multiple locations
NODE_CMD=""
NPM_CMD=""

# First, try common system locations (works for both sudo and non-sudo)
# Also check WSL-specific locations (nvm4w, etc.)
if command -v npm &> /dev/null; then
    # Try current PATH first (works when not using sudo)
    NPM_CMD="npm"
    NODE_CMD="node"
elif [ -f "/usr/bin/npm" ]; then
    NPM_CMD="/usr/bin/npm"
    NODE_CMD="/usr/bin/node"
elif [ -f "/usr/local/bin/npm" ]; then
    NPM_CMD="/usr/local/bin/npm"
    NODE_CMD="/usr/local/bin/node"
elif [ -f "/mnt/c/nvm4w/nodejs/npm" ]; then
    # WSL: nvm4w (Node Version Manager for Windows)
    NPM_CMD="/mnt/c/nvm4w/nodejs/npm"
    NODE_CMD="/mnt/c/nvm4w/nodejs/node"
fi

# When running with sudo, also try to find npm as the original user
if [ -z "$NPM_CMD" ] && [ -n "$SUDO_USER" ]; then
    # Try to find npm in the user's environment
    NPM_CMD=$(sudo -u "$ORIGINAL_USER" -i bash -c 'command -v npm' 2>/dev/null || echo "")
    NODE_CMD=$(sudo -u "$ORIGINAL_USER" -i bash -c 'command -v node' 2>/dev/null || echo "")
    
    # If still not found, try common locations
    if [ -z "$NPM_CMD" ]; then
        if [ -f "$ORIGINAL_HOME/.nvm/current/bin/npm" ]; then
            NPM_CMD="$ORIGINAL_HOME/.nvm/current/bin/npm"
            NODE_CMD="$ORIGINAL_HOME/.nvm/current/bin/node"
        elif [ -d "$ORIGINAL_HOME/.nvm/versions/node" ]; then
            # Try to find latest node version
            LATEST_NODE=$(ls -1 "$ORIGINAL_HOME/.nvm/versions/node" 2>/dev/null | sort -V | tail -1)
            if [ -n "$LATEST_NODE" ] && [ -f "$ORIGINAL_HOME/.nvm/versions/node/$LATEST_NODE/bin/npm" ]; then
                NPM_CMD="$ORIGINAL_HOME/.nvm/versions/node/$LATEST_NODE/bin/npm"
                NODE_CMD="$ORIGINAL_HOME/.nvm/versions/node/$LATEST_NODE/bin/node"
            fi
        fi
    fi
fi

# Final check - try to find in PATH if still not found
if [ -z "$NPM_CMD" ]; then
    # Try all common locations including WSL paths
    for npm_path in /usr/bin/npm /usr/local/bin/npm /opt/nodejs/bin/npm "$ORIGINAL_HOME/.local/bin/npm" /mnt/c/nvm4w/nodejs/npm /mnt/c/Program\ Files/nodejs/npm /mnt/c/Program\ Files\ \(x86\)/nodejs/npm; do
        if [ -f "$npm_path" ] && [ -x "$npm_path" ]; then
            NPM_CMD="$npm_path"
            # Find corresponding node
            node_path="${npm_path%/*}/node"
            if [ -f "$node_path" ]; then
                NODE_CMD="$node_path"
            fi
            break
        fi
    done
    
    # Last resort: use which/whereis if available (but only if not using sudo, as sudo changes PATH)
    if [ -z "$NPM_CMD" ] && [ -z "$SUDO_USER" ]; then
        if command -v which &> /dev/null; then
            NPM_CMD=$(which npm 2>/dev/null || echo "")
            NODE_CMD=$(which node 2>/dev/null || echo "")
        elif command -v whereis &> /dev/null; then
            npm_whereis=$(whereis -b npm 2>/dev/null | awk '{print $2}')
            node_whereis=$(whereis -b node 2>/dev/null | awk '{print $2}')
            if [ -n "$npm_whereis" ] && [ -x "$npm_whereis" ]; then
                NPM_CMD="$npm_whereis"
            fi
            if [ -n "$node_whereis" ] && [ -x "$node_whereis" ]; then
                NODE_CMD="$node_whereis"
            fi
        fi
    fi
fi

if [ -z "$NODE_CMD" ] || [ -z "$NPM_CMD" ]; then
    echo "❌ Node.js/npm not found. Please install Node.js first."
    echo ""
    echo "   Searched locations:"
    echo "   - /usr/bin/npm"
    echo "   - /usr/local/bin/npm"
    echo "   - /mnt/c/nvm4w/nodejs/npm (WSL)"
    echo "   - /mnt/c/Program Files/nodejs/npm (WSL)"
    echo "   - Current PATH: $PATH"
    if [ -n "$SUDO_USER" ]; then
        echo "   - User PATH (via sudo -u $ORIGINAL_USER -i)"
        echo "   - ~/.nvm/current/bin/npm"
        echo "   - ~/.nvm/versions/node/*/bin/npm"
        echo ""
        echo "   To debug, try running:"
        echo "   which npm"
        echo "   sudo -u $ORIGINAL_USER -i bash -c 'which npm'"
    fi
    exit 1
fi

# Verify versions
if [ -n "$SUDO_USER" ]; then
    NODE_VERSION=$(sudo -u "$ORIGINAL_USER" -i bash -c "'$NODE_CMD' --version" 2>/dev/null || echo "unknown")
    NPM_VERSION=$(sudo -u "$ORIGINAL_USER" -i bash -c "'$NPM_CMD' --version" 2>/dev/null || echo "unknown")
else
    NODE_VERSION=$($NODE_CMD --version 2>/dev/null || echo "unknown")
    NPM_VERSION=$($NPM_CMD --version 2>/dev/null || echo "unknown")
fi

echo "✅ Found Node.js: $NODE_CMD ($NODE_VERSION)"
echo "✅ Found npm: $NPM_CMD ($NPM_VERSION)"

# Build Rust agent
echo "🔨 Building Rust agent..."
# sqlx needs a database for compile-time verification (unless using offline mode)
# If DATABASE_URL is set, sqlx will use it; otherwise it will try to connect to default
if ! cargo build --release --workspace; then
    echo "❌ Failed to build Rust agent. Please check the error above."
    exit 1
fi

# Build TypeScript CLI
echo "🔨 Building TypeScript CLI..."
cd "$PROJECT_DIR/cli-ts"

# Use the found npm command, or run as original user if sudo
if [ -n "$SUDO_USER" ]; then
    echo "   Running npm as user: $ORIGINAL_USER"
    # Use -i to get a login shell with full environment
    if ! sudo -u "$ORIGINAL_USER" -i bash -c "cd '$PROJECT_DIR/cli-ts' && '$NPM_CMD' install"; then
        echo "❌ Failed to install npm dependencies. Please check the error above."
        exit 1
    fi
    if ! sudo -u "$ORIGINAL_USER" -i bash -c "cd '$PROJECT_DIR/cli-ts' && '$NPM_CMD' run build"; then
        echo "❌ Failed to build TypeScript CLI. Please check the error above."
        exit 1
    fi
else
    if ! $NPM_CMD install; then
        echo "❌ Failed to install npm dependencies. Please check the error above."
        exit 1
    fi
    if ! $NPM_CMD run build; then
        echo "❌ Failed to build TypeScript CLI. Please check the error above."
        exit 1
    fi
fi

# Create AppDir structure
echo "📁 Creating AppDir structure..."
mkdir -p "$APPIMAGE_DIR/usr/bin"
mkdir -p "$APPIMAGE_DIR/usr/lib/sia"
mkdir -p "$APPIMAGE_DIR/usr/share/sia"
mkdir -p "$APPIMAGE_DIR/usr/share/applications"
mkdir -p "$APPIMAGE_DIR/usr/share/icons/hicolor/256x256/apps"

# Copy agent binary
cp "$PROJECT_DIR/target/release/sia-agent" "$APPIMAGE_DIR/usr/bin/"

# Copy CLI
cp -r "$PROJECT_DIR/cli-ts/dist" "$APPIMAGE_DIR/usr/lib/sia/cli-ts/"
cp "$PROJECT_DIR/cli-ts/package.json" "$APPIMAGE_DIR/usr/lib/sia/cli-ts/"
cd "$APPIMAGE_DIR/usr/lib/sia/cli-ts"

# Install production dependencies
if [ -n "$SUDO_USER" ]; then
    sudo -u "$ORIGINAL_USER" -i bash -c "cd '$APPIMAGE_DIR/usr/lib/sia/cli-ts' && '$NPM_CMD' install --production --no-save"
else
    $NPM_CMD install --production --no-save
fi

# Create CLI wrapper
cat > "$APPIMAGE_DIR/usr/bin/sia-cli" <<'EOF'
#!/bin/bash
APPIMAGE_DIR="$(dirname "$(readlink -f "$0")")/../.."
cd "$APPIMAGE_DIR/usr/lib/sia/cli-ts"
node dist/index.js "$@"
EOF
chmod +x "$APPIMAGE_DIR/usr/bin/sia-cli"

# Copy config
cp -r "$PROJECT_DIR/config" "$APPIMAGE_DIR/usr/share/sia/"

# Create desktop entry - AppImageTool expects it in AppDir root AND usr/share/applications
cat > "$APPIMAGE_DIR/sia.desktop" <<EOF
[Desktop Entry]
Name=SIA - System Insight Agent
Comment=Local-first system monitoring and analysis tool
Exec=sia-cli
Icon=sia
Type=Application
Categories=System;Monitor;
Terminal=true
EOF

# Also create in standard location
cat > "$APPIMAGE_DIR/usr/share/applications/sia.desktop" <<EOF
[Desktop Entry]
Name=SIA - System Insight Agent
Comment=Local-first system monitoring and analysis tool
Exec=sia-cli
Icon=sia
Type=Application
Categories=System;Monitor;
Terminal=true
EOF

# Create AppRun
cat > "$APPIMAGE_DIR/AppRun" <<'EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "${0}")")"
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"

# Set config path
export SIA_CONFIG="${HERE}/usr/share/sia/config/default.toml"

# Run CLI if no args, otherwise pass args
if [ $# -eq 0 ]; then
    exec "${HERE}/usr/bin/sia-cli"
else
    exec "${HERE}/usr/bin/sia-cli" "$@"
fi
EOF
chmod +x "$APPIMAGE_DIR/AppRun"

# Download AppImageTool if not present
APPIMAGETOOL="$BUILD_DIR/appimagetool"
APPIMAGETOOL_EXTRACTED="$BUILD_DIR/appimagetool-extracted"
APPIMAGETOOL_BINARY="$BUILD_DIR/appimagetool-binary"

# Check if we're in Docker or FUSE is not available - use static binary instead
USE_STATIC_BINARY=false
if [ ! -f "$APPIMAGETOOL_BINARY" ] && [ ! -f "$APPIMAGETOOL_EXTRACTED/AppRun" ]; then
    if [ ! -e /dev/fuse ] || [ -f /.dockerenv ]; then
        USE_STATIC_BINARY=true
        echo "📥 Downloading AppImageTool (static binary for Docker)..."
        # Try to download a static binary version (if available) or we'll extract the AppImage
        if ! wget -q -O "$APPIMAGETOOL_BINARY" https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage 2>/dev/null || [ ! -x "$APPIMAGETOOL_BINARY" ]; then
            USE_STATIC_BINARY=false
        fi
    fi
    
    if [ "$USE_STATIC_BINARY" = false ]; then
        if [ ! -f "$APPIMAGETOOL" ]; then
            echo "📥 Downloading AppImageTool..."
            wget -q -O "$APPIMAGETOOL" https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
            chmod +x "$APPIMAGETOOL"
        fi
    fi
fi

# Extract AppImageTool if FUSE is not available (e.g., in Docker)
# In Docker, we always need to extract because docker build doesn't support --privileged
# The --appimage-help check might pass, but actual AppImage creation needs FUSE
NEEDS_EXTRACTION=false
if [ -f "$APPIMAGETOOL" ] && [ ! -f "$APPIMAGETOOL_EXTRACTED/AppRun" ]; then
    # In Docker build environment, always extract (docker build doesn't have --privileged)
    if [ -f /.dockerenv ]; then
        NEEDS_EXTRACTION=true
        echo "📦 Docker environment detected - will extract AppImageTool"
    elif [ ! -e /dev/fuse ]; then
        # FUSE device doesn't exist
        NEEDS_EXTRACTION=true
    else
        # Try to run it to see if FUSE actually works
        if "$APPIMAGETOOL" --appimage-help > /dev/null 2>&1; then
            # Test if we can actually create an AppImage (not just show help)
            TEST_DIR=$(mktemp -d)
            mkdir -p "$TEST_DIR/test.AppDir"
            echo "test" > "$TEST_DIR/test.AppDir/test.txt"
            if ARCH=x86_64 "$APPIMAGETOOL" "$TEST_DIR/test.AppDir" "$TEST_DIR/test.AppImage" > /dev/null 2>&1; then
                rm -rf "$TEST_DIR"
                echo "✅ AppImageTool works directly (FUSE available)"
                NEEDS_EXTRACTION=false
            else
                rm -rf "$TEST_DIR"
                NEEDS_EXTRACTION=true
            fi
        else
            NEEDS_EXTRACTION=true
        fi
    fi
    
    if [ "$NEEDS_EXTRACTION" = true ]; then
        echo "📦 Extracting AppImageTool (FUSE not available)..."
        # Try multiple extraction methods
        EXTRACTION_SUCCESS=false
        
        # Method 1: Try 7z (sometimes works with AppImages)
        if command -v 7z > /dev/null 2>&1 && [ "$EXTRACTION_SUCCESS" = false ]; then
            echo "   Trying 7z extraction..."
            cd "$BUILD_DIR"
            if 7z x "$APPIMAGETOOL" -o"$APPIMAGETOOL_EXTRACTED" > /dev/null 2>&1 && [ -f "$APPIMAGETOOL_EXTRACTED/AppRun" ]; then
                chmod +x "$APPIMAGETOOL_EXTRACTED/AppRun"
                # Also make sure other binaries are executable
                find "$APPIMAGETOOL_EXTRACTED" -type f -name "*.so*" -exec chmod +x {} \; 2>/dev/null || true
                find "$APPIMAGETOOL_EXTRACTED" -type f -path "*/usr/bin/*" -exec chmod +x {} \; 2>/dev/null || true
                echo "✅ AppImageTool extracted successfully using 7z"
                EXTRACTION_SUCCESS=true
            fi
        fi
        
        # Method 2: Try unsquashfs (works without FUSE)
        if command -v unsquashfs > /dev/null 2>&1 && [ "$EXTRACTION_SUCCESS" = false ]; then
            cd "$BUILD_DIR"
            # Find the squashfs offset by searching for magic bytes "hsqs" (0x68737173)
            # Use grep with binary mode to find the offset
            OFFSET=$(grep -abo "hsqs" "$APPIMAGETOOL" 2>/dev/null | head -1 | cut -d: -f1)
            
            # If grep -abo doesn't work, try using hexdump
            if [ -z "$OFFSET" ]; then
                HEX_OFFSET=$(hexdump -C "$APPIMAGETOOL" | grep -m1 "68 73 71 73" | awk '{print $1}')
                if [ -n "$HEX_OFFSET" ]; then
                    # Convert hex to decimal (remove leading zeros if any)
                    OFFSET=$((0x$HEX_OFFSET))
                fi
            fi
            
            if [ -n "$OFFSET" ] && [ "$OFFSET" -gt 0 ]; then
                echo "   Found squashfs at offset $OFFSET"
                # Try extracting with offset first
                if unsquashfs -d "$APPIMAGETOOL_EXTRACTED" -o "$OFFSET" "$APPIMAGETOOL" > /dev/null 2>&1; then
                    echo "✅ AppImageTool extracted successfully using unsquashfs at offset $OFFSET"
                else
                    # If that fails, extract the squashfs section to a temp file and extract that
                    echo "   Trying alternative extraction method..."
                    TEMP_SQUASHFS="$BUILD_DIR/temp_squashfs"
                    dd if="$APPIMAGETOOL" of="$TEMP_SQUASHFS" bs=1 skip="$OFFSET" 2>/dev/null
                    UNSQUASHFS_OUTPUT=$(unsquashfs -d "$APPIMAGETOOL_EXTRACTED" "$TEMP_SQUASHFS" 2>&1)
                    UNSQUASHFS_EXIT=$?
                    if [ $UNSQUASHFS_EXIT -eq 0 ]; then
                        rm -f "$TEMP_SQUASHFS"
                        echo "✅ AppImageTool extracted successfully using dd+unsquashfs"
                    else
                        # Try with offset -1 (sometimes the magic is one byte before the actual start)
                        echo "   Trying offset adjustment..."
                        rm -f "$TEMP_SQUASHFS"
                        ADJUSTED_OFFSET=$((OFFSET - 1))
                        dd if="$APPIMAGETOOL" of="$TEMP_SQUASHFS" bs=1 skip="$ADJUSTED_OFFSET" 2>/dev/null
                        if unsquashfs -d "$APPIMAGETOOL_EXTRACTED" "$TEMP_SQUASHFS" > /dev/null 2>&1; then
                            rm -f "$TEMP_SQUASHFS"
                            echo "✅ AppImageTool extracted successfully with adjusted offset"
                        else
                            rm -f "$TEMP_SQUASHFS"
                            echo "❌ unsquashfs failed: $UNSQUASHFS_OUTPUT"
                            echo "   Offset found: $OFFSET, but extraction failed"
                            exit 1
                        fi
                    fi
                fi
            else
                # Try without offset - some AppImages might work
                echo "   Trying direct extraction (no offset)..."
                if unsquashfs -d "$APPIMAGETOOL_EXTRACTED" "$APPIMAGETOOL" > /dev/null 2>&1; then
                    echo "✅ AppImageTool extracted successfully using unsquashfs"
                    EXTRACTION_SUCCESS=true
                fi
            fi
            
            if [ "$EXTRACTION_SUCCESS" = false ]; then
                echo "❌ All extraction methods failed"
                echo "   Tried: 7z, unsquashfs with offset finding"
                echo "   This might indicate the AppImage format is not supported"
                exit 1
            fi
        # Method 3: Use AppImageTool's own extraction feature (works without external tools)
        elif [ "$EXTRACTION_SUCCESS" = false ] && [ -f "$APPIMAGETOOL" ]; then
            echo "   Trying AppImageTool self-extraction..."
            cd "$BUILD_DIR"
            # AppImageTool can extract itself using --appimage-extract
            if "$APPIMAGETOOL" --appimage-extract > /dev/null 2>&1 && [ -f "squashfs-root/AppRun" ]; then
                mv squashfs-root "$APPIMAGETOOL_EXTRACTED"
                chmod +x "$APPIMAGETOOL_EXTRACTED/AppRun"
                echo "✅ AppImageTool extracted successfully using self-extraction"
                EXTRACTION_SUCCESS=true
            fi
        fi
        
        if [ "$EXTRACTION_SUCCESS" = false ]; then
            echo "❌ No extraction methods available (unsquashfs, 7z, or AppImage self-extraction)."
            echo "   Cannot extract AppImageTool without FUSE or extraction tools."
            echo "   Please install squashfs-tools: sudo pacman -S squashfs-tools"
            exit 1
        fi
    fi
fi

# Build AppImage
echo "🔨 Creating AppImage..."
cd "$BUILD_DIR"

# AppImageTool expects the desktop file to be named after the AppDir
# So if AppDir is "sia.AppDir", desktop should be "sia.desktop" in the root
if [ ! -f "$APPIMAGE_DIR/sia.desktop" ]; then
    echo "❌ Desktop file not found at $APPIMAGE_DIR/sia.desktop"
    exit 1
fi

# Use the appropriate AppImageTool version
# Ensure we're in the build directory and remove any existing output
cd "$BUILD_DIR"
rm -f sia-x86_64.AppImage

# Use relative paths (appimagetool prefers this)
APPIMAGE_INPUT="sia.AppDir"
APPIMAGE_OUTPUT="sia-x86_64.AppImage"

if [ -f "$APPIMAGETOOL_BINARY" ] && [ -x "$APPIMAGETOOL_BINARY" ]; then
    # Try to use static binary directly (if it's actually a binary and not an AppImage)
    if file "$APPIMAGETOOL_BINARY" | grep -q "ELF"; then
        ARCH=x86_64 "$APPIMAGETOOL_BINARY" "$APPIMAGE_INPUT" "$APPIMAGE_OUTPUT"
    else
        # It's still an AppImage, need to extract
        APPIMAGETOOL="$APPIMAGETOOL_BINARY"
        USE_STATIC_BINARY=false
    fi
elif [ -f "$APPIMAGETOOL_EXTRACTED/AppRun" ]; then
    # Set library path for extracted AppImage
    export LD_LIBRARY_PATH="$APPIMAGETOOL_EXTRACTED/usr/lib:$APPIMAGETOOL_EXTRACTED/usr/lib/x86_64-linux-gnu:$LD_LIBRARY_PATH"
    # Run appimagetool with explicit output filename
    # Capture exit code without triggering set -e
    set +e
    ARCH=x86_64 "$APPIMAGETOOL_EXTRACTED/AppRun" "$APPIMAGE_INPUT" "$APPIMAGE_OUTPUT" 2>&1
    APPIMAGETOOL_EXIT=$?
    set -e
    
    # Check if the AppImage was created (even if exit code is non-zero)
    if [ -f "$APPIMAGE_OUTPUT" ]; then
        echo "✅ AppImage created successfully: $APPIMAGE_OUTPUT"
        # Verify it's a valid AppImage
        if file "$APPIMAGE_OUTPUT" | grep -q "AppImage\|ELF"; then
            echo "✅ AppImage file is valid"
        fi
    elif [ $APPIMAGETOOL_EXIT -ne 0 ]; then
        echo "⚠️  AppImageTool failed (likely needs FUSE), trying manual AppImage creation..."
        # Fallback: Create AppImage manually using mksquashfs
        if command -v mksquashfs > /dev/null 2>&1; then
            echo "   Creating AppImage manually with mksquashfs..."
            # Download AppImage runtime if not present
            RUNTIME="$BUILD_DIR/runtime"
            if [ ! -f "$RUNTIME" ]; then
                echo "   Downloading AppImage runtime..."
                wget -q -O "$RUNTIME" https://github.com/AppImage/AppImageKit/releases/download/continuous/runtime-x86_64 || {
                    echo "❌ Failed to download AppImage runtime"
                    exit 1
                }
                chmod +x "$RUNTIME"
            fi
            
            # Create temporary squashfs
            TEMP_SQUASHFS="$BUILD_DIR/temp.squashfs"
            mksquashfs "$APPIMAGE_INPUT" "$TEMP_SQUASHFS" -root-owned -noappend || {
                echo "❌ mksquashfs failed"
                exit 1
            }
            
            # Combine runtime + squashfs into AppImage
            cat "$RUNTIME" "$TEMP_SQUASHFS" > "$APPIMAGE_OUTPUT"
            chmod +x "$APPIMAGE_OUTPUT"
            rm -f "$TEMP_SQUASHFS"
            
            if [ -f "$APPIMAGE_OUTPUT" ]; then
                echo "✅ AppImage created manually: $APPIMAGE_OUTPUT"
                if file "$APPIMAGE_OUTPUT" | grep -q "AppImage\|ELF"; then
                    echo "✅ AppImage file is valid"
                fi
            else
                echo "❌ Manual AppImage creation failed"
                exit 1
            fi
        else
            echo "❌ AppImageTool failed and mksquashfs not available"
            echo "   Exit code: $APPIMAGETOOL_EXIT"
            exit 1
        fi
    fi
elif [ -f "$APPIMAGETOOL" ]; then
    ARCH=x86_64 "$APPIMAGETOOL" "$APPIMAGE_INPUT" "$APPIMAGE_OUTPUT"
else
    echo "❌ AppImageTool not found"
    exit 1
fi

echo ""
echo "✅ AppImage built successfully!"
echo "📦 Location: $BUILD_DIR/sia-x86_64.AppImage"
echo ""
echo "To use:"
echo "  chmod +x $BUILD_DIR/sia-x86_64.AppImage"
echo "  ./$BUILD_DIR/sia-x86_64.AppImage"

