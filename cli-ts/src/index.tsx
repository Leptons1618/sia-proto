#!/usr/bin/env node

import React, { useState, useEffect, useMemo, useCallback, useRef } from 'react';
import { render, Text, Box, useInput, useApp, useStdout, type Key } from 'ink';
import { sendRequest, type IpcResponse } from './ipc.js';
import { showGreeting, printStatus, printList, printShow, printAnalyze, printConfig, showHelp } from './output.js';

const COMMANDS = ['status', 'stop', 'config', 'list', 'show', 'analyze', 'help', 'exit', 'quit'];

// Helper function to center text
function centerText(text: string, width: number): string {
  const textLen = text.length;
  const padding = Math.max(0, Math.floor((width - textLen) / 2));
  return ' '.repeat(padding) + text;
}

interface AppProps {
  args?: string[];
}

function App({ args }: AppProps) {
  // Command mode (non-interactive)
  if (args && args.length > 0) {
    return <CommandMode args={args} />;
  }

  // Interactive mode
  return <InteractiveMode />;
}

function CommandMode({ args }: { args: string[] }) {
  const { exit } = useApp();
  const [output, setOutput] = useState<string>('');
  const [error, setError] = useState<string>('');

  useEffect(() => {
    (async () => {
      try {
        const cmd = args[0];
        switch (cmd) {
          case 'status':
          case '/status': {
            const response = await sendRequest('status', undefined, undefined);
            setOutput(printStatus(response, 80));
            break;
          }
          case 'list':
          case '/list': {
            const limit = args[1] ? parseInt(args[1], 10) : 20;
            const response = await sendRequest('list', limit, undefined);
            setOutput(printList(response, 80));
            break;
          }
          case 'show':
          case '/show': {
            if (args[1]) {
              const response = await sendRequest('show', undefined, args[1]);
              setOutput(printShow(response, 80));
            } else {
              setError('❌ Usage: show <event_id>');
            }
            break;
          }
          case 'analyze':
          case '/analyze': {
            if (args[1]) {
              const response = await sendRequest('analyze', undefined, args[1]);
              setOutput(printAnalyze(response, 80));
            } else {
              setError('❌ Usage: analyze <event_id>');
            }
            break;
          }
          case 'stop':
          case '/stop': {
            setError('Stop command only works in interactive mode');
            break;
          }
          case 'help':
          case '/help':
          case '-h':
          case '--help': {
            setOutput(showHelp());
            break;
          }
          default: {
            setError(`❌ Unknown command: ${cmd}\nRun 'sia-cli-ts help' for usage information`);
          }
        }
      } catch (err) {
        setError(`❌ Error: ${err instanceof Error ? err.message : String(err)}`);
      } finally {
        setTimeout(() => exit(), 100);
      }
    })();
  }, []);

  if (error) {
    return <Text color="red">{error}</Text>;
  }

  return <Text>{output}</Text>;
}

// Memoized status component to prevent unnecessary re-renders
const StatusDisplay = React.memo(({ statusData, width }: { statusData: string; width: number }) => {
  return <Text>{statusData}</Text>;
}, (prevProps, nextProps) => {
  // Only re-render if statusData string actually changed
  return prevProps.statusData === nextProps.statusData && prevProps.width === nextProps.width;
});

// Memoized greeting/logo component - never re-renders
const GreetingLogo = React.memo(() => {
  return (
    <Box flexDirection="column">
      <Text>{'\n'}</Text>
      {/* Pixelated "System Insight Agent" logo, left-aligned */}
      <Text color="cyan" bold>
        {' '}███████╗██╗   ██╗███████╗████████╗███████╗███╗   ███╗
      </Text>
      <Text color="cyan" bold>
        {' '}██╔════╝╚██╗ ██╔╝██╔════╝╚══██╔══╝██╔════╝████╗ ████║
      </Text>
      <Text color="cyan" bold>
        {' '}███████╗ ╚████╔╝ ███████╗   ██║   █████╗  ██╔████╔██║
      </Text>
      <Text color="cyan" bold>
        {' '}╚════██║  ╚██╔╝  ╚════██║   ██║   ██╔══╝  ██║╚██╔╝██║
      </Text>
      <Text color="cyan" bold>
        {' '}███████║   ██║   ███████║   ██║   ███████╗██║ ╚═╝ ██║
      </Text>
      <Text color="cyan" bold>
        {' '}╚══════╝   ╚═╝   ╚══════╝   ╚═╝   ╚══════╝╚═╝     ╚═╝
      </Text>
      <Text>{'\n'}</Text>
      <Text color="cyan" bold>
        {' '}██╗███╗   ██╗███████╗██╗ ██████╗ ██╗   ██╗███████╗██████╗ ████████╗
      </Text>
      <Text color="cyan" bold>
        {' '}██║████╗  ██║██╔════╝██║██╔═══██╗██║   ██║██╔════╝██╔══██╗╚══██╔══╝
      </Text>
      <Text color="cyan" bold>
        {' '}██║██╔██╗ ██║███████╗██║██║   ██║██║   ██║█████╗  ██████╔╝   ██║
      </Text>
      <Text color="cyan" bold>
        {' '}██║██║╚██╗██║╚════██║██║██║   ██║╚██╗ ██╔╝██╔══╝  ██╔══██╗   ██║
      </Text>
      <Text color="cyan" bold>
        {' '}██║██║ ╚████║███████║██║╚██████╔╝ ╚████╔╝ ███████╗██║  ██║   ██║
      </Text>
      <Text color="cyan" bold>
        {' '}╚═╝╚═╝  ╚═══╝╚══════╝╚═╝ ╚═════╝   ╚═══╝  ╚══════╝╚═╝  ╚═╝   ╚═╝
      </Text>
      <Text>{'\n'}</Text>
      <Text color="cyan" bold>
        {' '} █████╗  ██████╗ ███████╗███╗   ██╗████████╗
      </Text>
      <Text color="cyan" bold>
        {' '}██╔══██╗██╔════╝ ██╔════╝████╗  ██║╚══██╔══╝
      </Text>
      <Text color="cyan" bold>
        {' '}███████║██║  ███╗█████╗  ██╔██╗ ██║   ██║
      </Text>
      <Text color="cyan" bold>
        {' '}██╔══██║██║   ██║██╔══╝  ██║╚██╗██║   ██║
      </Text>
      <Text color="cyan" bold>
        {' '}██║  ██║╚██████╔╝███████╗██║ ╚████║   ██║
      </Text>
      <Text color="cyan" bold>
        {' '}╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═╝  ╚═══╝   ╚═╝
      </Text>
      <Text>{'\n'}</Text>
      <Text color="white" dimColor>
        {' '}A local-first system monitoring and analysis tool with CPU/memory monitoring, event analysis, and LLM integration
      </Text>
      <Text>{'\n'}</Text>
      <Text color="white" dimColor>
        {' '}Type /help for available commands
      </Text>
    </Box>
  );
}, () => true); // Never re-render

// Cursor component - completely isolated, uses ref to minimize re-renders
// Only updates when absolutely necessary
const BlinkingCursor = ({ shouldBlink }: { shouldBlink: boolean }) => {
  const [visible, setVisible] = React.useState(true);
  const intervalRef = React.useRef<NodeJS.Timeout | null>(null);
  const mountedRef = React.useRef(true);
  
  React.useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, []);
  
  React.useEffect(() => {
    // Clear any existing interval
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
    
    if (!shouldBlink) {
      if (!visible && mountedRef.current) {
        setVisible(true);
      }
      return;
    }
    
    // Only start blinking if we should
    intervalRef.current = setInterval(() => {
      if (mountedRef.current) {
        setVisible(prev => !prev);
      }
    }, 800); // Slower blink to reduce flicker
    
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, [shouldBlink]); // Only depend on shouldBlink, not visible
  
  // Always show cursor - blinking is optional and less frequent
  return <Text inverse={visible}> </Text>;
};

// Memoized input box component - static structure, cursor is separate
const InputBox = React.memo(({ 
  prompt, 
  displayInput, 
  shouldBlink,
  remaining, 
  boxWidth 
}: { 
  prompt: string; 
  displayInput: string; 
  shouldBlink: boolean;
  remaining: number; 
  boxWidth: number;
}) => {
  // Memoize the box structure to prevent re-renders
  const topBorder = useMemo(() => '┌' + '─'.repeat(boxWidth) + '┐', [boxWidth]);
  const bottomBorder = useMemo(() => '└' + '─'.repeat(boxWidth) + '┘', [boxWidth]);
  const padding = useMemo(() => ' '.repeat(remaining), [remaining]);
  
  return (
    <Box flexDirection="column" marginTop={1}>
      <Text color="cyan">{topBorder}</Text>
      <Box>
        <Text color="cyan">│ </Text>
        <Text color="cyan" bold>{prompt}</Text>
        <Text color="white">{displayInput}</Text>
        <BlinkingCursor shouldBlink={shouldBlink} />
        <Text>{padding}</Text>
        <Text color="cyan">│</Text>
      </Box>
      <Text color="cyan">{bottomBorder}</Text>
    </Box>
  );
}, (prevProps, nextProps) => {
  // Only re-render if input, box width, or blink state changed
  // Cursor visibility is handled by BlinkingCursor component internally
  return prevProps.displayInput === nextProps.displayInput &&
         prevProps.boxWidth === nextProps.boxWidth &&
         prevProps.shouldBlink === nextProps.shouldBlink;
});

// Memoized connection status component with proper comparison
const ConnectionStatus = React.memo(({ status }: { status: {backend: boolean; llm: {available: boolean; model: string | null}} }) => {
  return (
    <Box flexDirection="row" marginTop={1} gap={2}>
      <Text dimColor>
        Backend: {status.backend ? (
          <Text color="green">● Connected</Text>
        ) : (
          <Text color="red">● Disconnected</Text>
        )}
      </Text>
      {status.llm.model ? (
        <Text dimColor>
          {' | '}AI Model: <Text color={status.llm.available ? "cyan" : "yellow"}>{status.llm.model}</Text>
          {!status.llm.available && <Text color="yellow" dimColor> (disconnected)</Text>}
        </Text>
      ) : (
        <Text dimColor>
          {' | '}AI Model: <Text color="yellow">Not configured</Text>
        </Text>
      )}
    </Box>
  );
}, (prevProps, nextProps) => {
  // Only re-render if status actually changed
  return JSON.stringify(prevProps.status) === JSON.stringify(nextProps.status);
});

function InteractiveMode() {
  const { exit } = useApp();
  const { stdout } = useStdout();
  const [input, setInput] = useState('');
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [selectedSuggestion, setSelectedSuggestion] = useState(0);
  const [allFilteredCommands, setAllFilteredCommands] = useState<string[]>([]);
  const [suggestionOffset, setSuggestionOffset] = useState(0);
  const [outputHistory, setOutputHistory] = useState<string[]>([]);
  const [currentStatus, setCurrentStatus] = useState<string>('');
  const [showGreetingText, setShowGreetingText] = useState(true);
  const [isLiveMode, setIsLiveMode] = useState(false);
  const [liveInterval, setLiveInterval] = useState<NodeJS.Timeout | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<{backend: boolean; llm: {available: boolean; model: string | null}}>({
    backend: false,
    llm: { available: false, model: null }
  });
  const [historyScrollOffset, setHistoryScrollOffset] = useState(0);
  const [isScrollingHistory, setIsScrollingHistory] = useState(false);
  const MAX_VISIBLE_HISTORY = 10; // Maximum number of history items to show at once
  
  // Get terminal width, default to 80 if not available - memoize to prevent re-renders
  const terminalWidth = useMemo(() => stdout.columns || 80, [stdout.columns]);
  const boxWidth = useMemo(() => Math.max(Math.floor((terminalWidth - 4) * 0.8), 50), [terminalWidth]);

  useInput((char: string, key: Key) => {
    if (key.ctrl && char === 'c') {
      exit();
      return;
    }

    if (key.return) {
      const cmd = suggestions.length > 0 && selectedSuggestion < suggestions.length
        ? suggestions[selectedSuggestion]
        : input.trim();

      if (cmd) {
        handleCommand(cmd.trimStart().replace(/^\//, ''));
      }

      setInput('');
      setSuggestions([]);
      setSelectedSuggestion(0);
      setShowGreetingText(false);
      return;
    }

    if (key.upArrow) {
      if (suggestions.length > 0) {
        const newIndex = selectedSuggestion - 1;
        if (newIndex < 0 && suggestionOffset > 0) {
          // Scroll suggestions up
          const newOffset = suggestionOffset - 1;
          setSuggestionOffset(newOffset);
          setSuggestions(allFilteredCommands.slice(newOffset, newOffset + 3));
          setSelectedSuggestion(0);
        } else if (newIndex >= 0) {
          setSelectedSuggestion(newIndex);
        }
      } else if (outputHistory.length > 0 && (key.shift || isScrollingHistory)) {
        // Shift+Up or when in scroll mode: scroll history up (show older items)
        setIsScrollingHistory(true);
        setHistoryScrollOffset(prev => {
          const maxOffset = Math.max(0, outputHistory.length - MAX_VISIBLE_HISTORY);
          return Math.min(prev + 1, maxOffset);
        });
      }
      return;
    }

    if (key.downArrow) {
      if (suggestions.length > 0) {
        const newIndex = selectedSuggestion + 1;
        if (newIndex >= suggestions.length && suggestionOffset + suggestions.length < allFilteredCommands.length) {
          // Scroll suggestions down
          const newOffset = suggestionOffset + 1;
          setSuggestionOffset(newOffset);
          setSuggestions(allFilteredCommands.slice(newOffset, newOffset + 3));
          setSelectedSuggestion(suggestions.length - 1);
        } else if (newIndex < suggestions.length) {
          setSelectedSuggestion(newIndex);
        }
      } else if (outputHistory.length > 0 && (key.shift || isScrollingHistory)) {
        // Shift+Down or when in scroll mode: scroll history down (show newer items)
        setIsScrollingHistory(true);
        setHistoryScrollOffset(prev => {
          const newOffset = Math.max(0, prev - 1);
          // Auto-exit scroll mode when reaching bottom
          if (newOffset === 0) {
            setIsScrollingHistory(false);
          }
          return newOffset;
        });
      }
      return;
    }
    
    // Exit scroll mode when typing
    if (isScrollingHistory && char) {
      setIsScrollingHistory(false);
    }

    if (key.tab && suggestions.length > 0 && selectedSuggestion < suggestions.length) {
      setInput(suggestions[selectedSuggestion]);
      setSuggestions([]);
      setSelectedSuggestion(0);
      return;
    }

    if (key.escape) {
      setSuggestions([]);
      setSelectedSuggestion(0);
      return;
    }

    if (key.backspace || key.delete) {
      // Always allow backspace, even if input is just "/"
      const newInput = input.length > 0 ? input.slice(0, -1) : '';
      setInput(newInput);
      updateSuggestions(newInput);
      return;
    }

    // Handle regular character input
    if (char) {
      const newInput = input + char;
      setInput(newInput);
      updateSuggestions(newInput);
    }
  });

  function updateSuggestions(text: string) {
    if (text.startsWith('/')) {
      const query = text.slice(1);
      const filtered = COMMANDS
        .filter(cmd => cmd.startsWith(query))
        .map(cmd => `/${cmd}`);
      // Store all filtered commands for scrolling
      setAllFilteredCommands(filtered);
      setSuggestionOffset(0);
      // Show first 3 suggestions
      const limited = filtered.slice(0, 3);
      setSuggestions(limited);
      setSelectedSuggestion(0);
    } else {
      setAllFilteredCommands([]);
      setSuggestionOffset(0);
      setSuggestions([]);
      setSelectedSuggestion(0);
    }
  }

  // Determine if cursor should blink - only when actively typing, not on main page
  // Disable blinking to prevent flicker - cursor will always be visible
  const shouldBlinkCursor = useMemo(() => {
    // Only blink when user is actively typing (has input), not on empty main page
    return false; // Disabled to prevent flickering - cursor always visible
    // return !isLiveMode && outputHistory.length === 0 && input.length > 0;
  }, [isLiveMode, outputHistory.length, input.length]);

  // Cleanup live mode on unmount
  useEffect(() => {
    return () => {
      if (liveInterval) {
        clearInterval(liveInterval);
      }
    };
  }, [liveInterval]);

  // Memoize status data to prevent unnecessary re-renders
  const statusDataRef = React.useRef<any>(null);
  
  async function fetchStatus() {
    try {
      const response = await sendRequest('status', undefined, undefined);
      
      // Only update if data actually changed (deep comparison of key metrics)
      const newData = response.success ? {
        cpu_usage: response.data?.metrics?.cpu_usage || 0,
        memory_percent: response.data?.metrics?.memory_percent || 0,
        memory_used_mb: response.data?.metrics?.memory_used_mb || 0,
        memory_total_mb: response.data?.metrics?.memory_total_mb || 0,
        uptime: response.data?.uptime_seconds || 0,
        events: response.data?.events || {},
        thresholds: response.data?.thresholds || null
      } : null;
      
      // Only update if data changed (compare all relevant fields)
      const dataChanged = !statusDataRef.current || 
        statusDataRef.current.cpu_usage !== newData?.cpu_usage ||
        statusDataRef.current.memory_percent !== newData?.memory_percent ||
        statusDataRef.current.memory_used_mb !== newData?.memory_used_mb ||
        statusDataRef.current.memory_total_mb !== newData?.memory_total_mb ||
        statusDataRef.current.uptime !== newData?.uptime ||
        JSON.stringify(statusDataRef.current.events) !== JSON.stringify(newData?.events) ||
        JSON.stringify(statusDataRef.current.thresholds) !== JSON.stringify(newData?.thresholds);
      
      if (dataChanged && newData) {
        statusDataRef.current = newData;
        const statusOutput = printStatus(response, terminalWidth);
        setCurrentStatus(statusOutput);
      }
      
      // Update connection status separately (only when it actually changes)
      if (response.success) {
        const newConnectionStatus = {
          backend: true,
          llm: {
            available: response.data?.llm?.available || false,
            model: response.data?.llm?.model || null
          }
        };
        
        // Only update connection status if it changed (use ref to avoid re-renders)
        setConnectionStatus(prev => {
          const prevStr = JSON.stringify(prev);
          const newStr = JSON.stringify(newConnectionStatus);
          if (prevStr !== newStr) {
            return newConnectionStatus;
          }
          return prev;
        });
      }
    } catch (err) {
      const errorMsg = `❌ Error: ${err instanceof Error ? err.message : String(err)}`;
      setCurrentStatus(prev => prev !== errorMsg ? errorMsg : prev);
      setConnectionStatus({
        backend: false,
        llm: { available: false, model: null }
      });
    }
  }

  // Check connection status on mount only (not periodically to avoid flicker)
  const connectionStatusRef = useRef(connectionStatus);
  useEffect(() => {
    connectionStatusRef.current = connectionStatus;
  }, [connectionStatus]);
  
  useEffect(() => {
    const checkConnection = async () => {
      try {
        const response = await sendRequest('status', undefined, undefined);
        if (response.success) {
          const newStatus = {
            backend: true,
            llm: {
              available: response.data?.llm?.available || false,
              model: response.data?.llm?.model || null
            }
          };
          // Only update if changed
          if (JSON.stringify(connectionStatusRef.current) !== JSON.stringify(newStatus)) {
            setConnectionStatus(newStatus);
          }
        } else {
          const newStatus = {
            backend: false,
            llm: { available: false, model: null }
          };
          if (JSON.stringify(connectionStatusRef.current) !== JSON.stringify(newStatus)) {
            setConnectionStatus(newStatus);
          }
        }
      } catch (err) {
        const newStatus = {
          backend: false,
          llm: { available: false, model: null }
        };
        if (JSON.stringify(connectionStatusRef.current) !== JSON.stringify(newStatus)) {
          setConnectionStatus(newStatus);
        }
      }
    };

    checkConnection();
    // Removed periodic check - connection status will update via fetchStatus in live mode
  }, []);

  async function handleCommand(cmd: string) {
    const parts = cmd.split(/\s+/);
    if (parts.length === 0) return;

    const command = parts[0];

    // Stop live mode if running
    if (liveInterval) {
      clearInterval(liveInterval);
      setLiveInterval(null);
      setIsLiveMode(false);
    }

    try {
      switch (command) {
        case 'status': {
          // Start live mode for status
          setIsLiveMode(true);
          await fetchStatus(); // Initial fetch
          
          // Set up live updates every 2 seconds (throttled to prevent flicker)
          const interval = setInterval(async () => {
            await fetchStatus();
          }, 2000);
          setLiveInterval(interval);
          // Reset scroll when starting new status
          setHistoryScrollOffset(0);
          setIsScrollingHistory(false);
          break;
        }
        case 'list': {
          const limit = parts[1] ? parseInt(parts[1], 10) : 20;
          const response = await sendRequest('list', limit, undefined);
          const output = printList(response, terminalWidth);
          setOutputHistory(prev => [...prev, output]);
          setHistoryScrollOffset(0); // Reset scroll to bottom
          setIsScrollingHistory(false);
          break;
        }
        case 'show': {
          if (parts[1]) {
            const response = await sendRequest('show', undefined, parts[1]);
            const output = printShow(response, terminalWidth);
            setOutputHistory(prev => [...prev, output]);
            setHistoryScrollOffset(0);
            setIsScrollingHistory(false);
          } else {
            setOutputHistory(prev => [...prev, '❌ Usage: show <event_id>']);
            setHistoryScrollOffset(0);
            setIsScrollingHistory(false);
          }
          break;
        }
        case 'analyze': {
          if (parts[1]) {
            setOutputHistory(prev => [...prev, '🤖 Analyzing event with LLM...']);
            const response = await sendRequest('analyze', undefined, parts[1]);
            const output = printAnalyze(response, terminalWidth);
            setOutputHistory(prev => [...prev.slice(0, -1), output]);
            setHistoryScrollOffset(0);
            setIsScrollingHistory(false);
          } else {
            setOutputHistory(prev => [...prev, '❌ Usage: analyze <event_id>']);
            setHistoryScrollOffset(0);
            setIsScrollingHistory(false);
          }
          break;
        }
        case 'stop': {
          if (liveInterval) {
            clearInterval(liveInterval);
            setLiveInterval(null);
            setIsLiveMode(false);
            setCurrentStatus('');
            setOutputHistory(prev => [...prev, 'Live mode stopped. Use /status to start again.']);
          } else {
            setOutputHistory(prev => [...prev, 'No live mode active.']);
          }
          break;
        }
        case 'config': {
          const response = await sendRequest('status', undefined, undefined);
          const output = printConfig(response, terminalWidth);
          setOutputHistory(prev => [...prev, output]);
          setHistoryScrollOffset(0);
          setIsScrollingHistory(false);
          break;
        }
        case 'help': {
          setOutputHistory(prev => [...prev, showHelp()]);
          break;
        }
        case 'exit':
        case 'quit':
        case 'q':
        case 'bye': {
          if (liveInterval) {
            clearInterval(liveInterval);
          }
          exit();
          break;
        }
        default: {
          setOutputHistory(prev => [...prev, `❌ Unknown command: ${command}\nType /help for available commands`]);
        }
      }
    } catch (err) {
      setOutputHistory(prev => [...prev, `❌ Error: ${err instanceof Error ? err.message : String(err)}`]);
    }
  }

  // Memoize input box calculations
  const prompt = '▶ ';
  const inputWidth = useMemo(() => boxWidth - prompt.length - 1, [boxWidth]); // -1 for cursor
  const displayInput = useMemo(() => input.length > inputWidth ? input.slice(-inputWidth) : input, [input, inputWidth]);
  const remaining = useMemo(() => Math.max(0, inputWidth - displayInput.length), [inputWidth, displayInput.length]);
  

  // Memoize greeting display - separate live mode indicator to prevent re-renders
  const greetingDisplay = useMemo(() => {
    if (!showGreetingText) return null;
    return (
      <Box flexDirection="column" width={terminalWidth}>
        <GreetingLogo />
        <Text>{'\n'}</Text>
      </Box>
    );
  }, [showGreetingText, terminalWidth]);
  
  // Separate live mode indicator
  const liveModeIndicator = useMemo(() => {
    if (!showGreetingText || !isLiveMode) return null;
    return (
      <Text color="yellow" dimColor>
        {' '}● Live mode active - Use /stop to exit
      </Text>
    );
  }, [showGreetingText, isLiveMode]);

  return (
    <Box flexDirection="column">
      {greetingDisplay}
      {liveModeIndicator}

      {/* Output history - scrollable, preserved commands */}
      {outputHistory.length > 0 && (
        <Box flexDirection="column" marginTop={1}>
          {outputHistory.length > MAX_VISIBLE_HISTORY && (
            <Text dimColor>
              {' '}📜 History: Showing {historyScrollOffset === 0 ? 'latest' : `${historyScrollOffset + 1}-${Math.min(historyScrollOffset + MAX_VISIBLE_HISTORY, outputHistory.length)}`} of {outputHistory.length} items. {isScrollingHistory ? 'Use Shift+↑/↓ to scroll' : 'Press Shift+↑ to scroll up'}
            </Text>
          )}
          {outputHistory
            .slice(historyScrollOffset, historyScrollOffset + MAX_VISIBLE_HISTORY)
            .map((output, idx) => (
              <Box key={`history-${historyScrollOffset + idx}`} flexDirection="column" marginTop={1}>
                <Text>{output}</Text>
                <Text>{'\n'}</Text>
              </Box>
            ))}
        </Box>
      )}

      {/* Live status display - only this re-renders in live mode, optimized */}
      {isLiveMode && currentStatus && (
        <Box flexDirection="column" marginTop={1} key="live-status">
          <StatusDisplay statusData={currentStatus} width={terminalWidth} />
        </Box>
      )}

      {/* Input box - memoized to prevent unnecessary re-renders, cursor blinking is internal */}
      <InputBox 
        prompt={prompt}
        displayInput={displayInput}
        shouldBlink={shouldBlinkCursor}
        remaining={remaining}
        boxWidth={boxWidth}
      />
      
      {/* Connection Status - memoized to prevent re-renders */}
      <ConnectionStatus status={connectionStatus} />

      {suggestions.length > 0 && (
        <Box flexDirection="column" marginTop={1}>
          <Text dimColor>  Commands (↑/↓ to navigate, Tab to select):</Text>
          {suggestions.map((sug, i) => (
            <Text key={i} color={i === selectedSuggestion ? 'cyan' : 'white'} bold={i === selectedSuggestion} dimColor={i !== selectedSuggestion}>
              {i === selectedSuggestion ? `  ▶ ${sug}` : `    ${sug}`}
            </Text>
          ))}
          {allFilteredCommands.length > 3 && (
            <Text dimColor>  ... (showing {suggestionOffset + 1}-{suggestionOffset + suggestions.length} of {allFilteredCommands.length} commands, use ↑/↓ to scroll)</Text>
          )}
        </Box>
      )}
    </Box>
  );
}

const args = process.argv.slice(2);
render(<App args={args} />);

