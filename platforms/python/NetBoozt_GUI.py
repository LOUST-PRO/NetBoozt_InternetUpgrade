#!/usr/bin/env python3
"""
╔═══════════════════════════════════════════════════════════════════════════════╗
║                          NetBoozt - GUI Application                           ║
║                     Network Optimization Tool for Windows                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║  Author:  LOUST (www.loust.pro)                                               ║
║  Version: 2.2.x                                                               ║
║  License: Apache-2.0                                                           ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║  Entry Point: GUI Application with System Tray                                ║
║                                                                               ║
║  Features:                                                                    ║
║  - Modern CustomTkinter interface                                             ║
║  - Real-time network monitoring                                               ║
║  - DNS Intelligence with parallel analysis                                    ║
║  - System tray icon with quick actions                                        ║
║  - 4-phase network diagnostics                                                ║
║  - TCP/IP optimizations (BBR-like)                                            ║
╚═══════════════════════════════════════════════════════════════════════════════╝

Usage:
    python NetBoozt_GUI.py              # Run GUI application
    NetBoozt_GUI.exe                    # Compiled executable

Build:
    nuitka --standalone --windows-console-mode=disable NetBoozt_GUI.py
"""

# Redirect to main launcher
import run_modern

if __name__ == "__main__":
    # run_modern.py handles everything including:
    # - Splash screen
    # - System tray integration
    # - Main window
    pass
