import { createConnection } from 'net';
import { existsSync } from 'fs';
import * as os from 'os';
import * as path from 'path';

export interface IpcRequest {
  method: string;
  limit?: number;
  event_id?: string;
}

export interface IpcResponse {
  success: boolean;
  data: any;
}

function findSocketPath(): string {
  // Check environment variable first
  const envSocket = process.env.SIA_SOCKET;
  if (envSocket && existsSync(envSocket)) {
    return envSocket;
  }
  
  // Platform-specific paths
  const platform = os.platform();
  
  if (platform === 'win32') {
    // Windows: Check common locations
    // On Windows, Unix sockets might be in WSL or we might need named pipes
    // For now, try common Unix socket locations that might work with WSL
    const possiblePaths = [
      path.join(os.tmpdir(), 'sia.sock'),
      '\\\\.\\pipe\\sia',
      '//./pipe/sia',
    ];
    
    for (const socketPath of possiblePaths) {
      // On Windows, we can't reliably check if a socket exists via existsSync
      // So we'll try to connect and let the connection error tell us
      return socketPath;
    }
    
    // Default Windows path
    return path.join(os.tmpdir(), 'sia.sock');
  } else {
    // Unix-like systems
    const unixPaths = [
      '/run/sia/sia.sock',
      '/tmp/sia.sock',
      path.join(os.tmpdir(), 'sia.sock'),
    ];
    
    for (const socketPath of unixPaths) {
      if (existsSync(socketPath)) {
        return socketPath;
      }
    }
    
    // If none found, return the most common default
    return '/tmp/sia.sock';
  }
}

export async function sendRequest(
  method: string,
  limit?: number,
  event_id?: string
): Promise<IpcResponse> {
  const socketPath = findSocketPath();
  
  return new Promise((resolve, reject) => {
    const client = createConnection(socketPath);
    let buffer = '';
    let connected = false;

    client.on('connect', () => {
      connected = true;
      const request: IpcRequest = {
        method,
        ...(limit !== undefined && { limit }),
        ...(event_id && { event_id }),
      };

      const requestJson = JSON.stringify(request);
      client.write(requestJson);
      client.end();
    });

    client.on('data', (data) => {
      buffer += data.toString();
    });

    client.on('end', () => {
      try {
        if (buffer.trim()) {
          const response: IpcResponse = JSON.parse(buffer);
          resolve(response);
        } else {
          reject(new Error('Empty response from agent'));
        }
      } catch (err) {
        reject(new Error(`Failed to parse response: ${err instanceof Error ? err.message : String(err)}`));
      }
    });

    client.on('error', (err) => {
      if (!connected) {
        // Connection failed - provide helpful error message
        const platform = os.platform();
        if (platform === 'win32') {
          reject(new Error(
            `Cannot connect to SIA agent at ${socketPath}.\n` +
            `Is the agent running? On Windows, make sure you're using WSL or a Unix-compatible environment.\n` +
            `You can set SIA_SOCKET environment variable to specify the socket path.\n` +
            `Original error: ${err.message}`
          ));
        } else {
          reject(new Error(
            `Cannot connect to SIA agent at ${socketPath}.\n` +
            `Is the agent running? Start it with: cargo run --bin sia-agent\n` +
            `You can set SIA_SOCKET environment variable to specify a different socket path.\n` +
            `Original error: ${err.message}`
          ));
        }
      } else {
        reject(new Error(`Socket error: ${err.message}`));
      }
    });
    
    // Set a timeout for connection
    setTimeout(() => {
      if (!connected) {
        client.destroy();
        reject(new Error(`Connection timeout. Is the agent running at ${socketPath}?`));
      }
    }, 5000);
  });
}

