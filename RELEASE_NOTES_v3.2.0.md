# Release v3.2.0 - Nuclear Network Cleanup

## Summary
This release performs a full nuclear cleanup of the AXIOM Protocol network stack. All legacy code and duplicate logic have been removed. The network stack is now fully modular, deduplicated, and production-ready, with a clean build and mainnet validation.

## Major Changes
- **Nuclear Cleanup**: Removed all legacy and duplicate code from network modules
- **behaviour.rs**: Rewritten from scratch, deduplicated, and aligned with libp2p 0.54
- **Build**: All modules compile cleanly with zero errors
- **Mainnet Validation**: Node runs and syncs on mainnet
- **README**: Updated with new version and summary

## Upgrade Instructions
- Build and run as before; node now uses the deduplicated, production network stack
- See `src/network/` for new code

## Contributors
- Ghost-84M
- khanssameer19-png

## Date
February 8, 2026
