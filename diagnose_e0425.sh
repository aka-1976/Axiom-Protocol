#!/bin/bash
# Axiom Diagnostic Script - Find and Fix E0425 Errors
# E0425 = "cannot find value/function/module in this scope"

echo "🔍 AXIOM DIAGNOSTIC - E0425 Error Hunter"
echo "=========================================="
echo ""

# Check environment
echo "📋 Environment Check:"
echo "   Rust version: $(rustc --version 2>/dev/null || echo 'NOT FOUND')"
echo "   Cargo version: $(cargo --version 2>/dev/null || echo 'NOT FOUND')"
echo "   Current directory: $(pwd)"
echo ""

if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Not in Axiom-Protocol directory"
    exit 1
fi

echo "🔍 Step 1: Clean build to get clear error messages..."
cargo clean > /dev/null 2>&1
echo ""

echo "🔍 Step 2: Attempting build with full error output..."
echo "   (This will show the actual E0425 error)"
echo ""

# Capture full error output
BUILD_OUTPUT=$(cargo build 2>&1)
EXIT_CODE=$?

# Save to file
echo "$BUILD_OUTPUT" > diagnostic_build.log

# Display relevant errors
echo "════════════════════════════════════════"
echo "$BUILD_OUTPUT" | grep -A 10 "error\[E0425\]" || echo "No E0425 errors found in current build"
echo "$BUILD_OUTPUT" | grep -A 10 "error\[E0412\]" || true  # E0412 = cannot find type
echo "$BUILD_OUTPUT" | grep -A 10 "error\[E0433\]" || true  # E0433 = cannot find module
echo "════════════════════════════════════════"
echo ""

# Common E0425 issues and solutions
echo "📚 Common E0425 Causes in Rust:"
echo ""
echo "1. Missing import/use statement"
echo "   Fix: Add 'use crate::module::Item;' at top of file"
echo ""
echo "2. Module not declared in lib.rs"
echo "   Fix: Add 'pub mod module_name;' in src/lib.rs"
echo ""
echo "3. Trying to use private item from another module"
echo "   Fix: Make item public with 'pub' keyword"
echo ""
echo "4. Typo in variable/function name"
echo "   Fix: Check spelling and capitalization"
echo ""
echo "5. Item defined in tests but used in main code"
echo "   Fix: Move item out of #[cfg(test)] block"
echo ""

# Check for common issues in the codebase
echo "🔍 Step 3: Checking for common issues..."
echo ""

# Check if ai module is properly declared
if [ -f "src/ai/mod.rs" ]; then
    if ! grep -q "pub mod ai;" src/lib.rs; then
        echo "⚠️  FOUND ISSUE: src/ai/mod.rs exists but not declared in lib.rs"
        echo "   FIX: Add 'pub mod ai;' to src/lib.rs"
        echo ""
    else
        echo "✓ AI module properly declared"
    fi
fi

# Check for binary files that might reference missing items
echo ""
echo "🔍 Step 4: Checking binary files..."
if [ -d "src/bin" ]; then
    echo "   Found binaries:"
    ls -1 src/bin/*.rs 2>/dev/null | while read bin; do
        echo "   - $(basename $bin)"
    done
fi
echo ""

# Look for specific patterns that cause E0425
echo "🔍 Step 5: Searching for potential issues..."

# Check for use statements with missing modules
grep -rn "use crate::" src/ 2>/dev/null | head -20 || true

echo ""
echo "📄 Full build log saved to: diagnostic_build.log"
echo ""

if [ $EXIT_CODE -ne 0 ]; then
    echo "❌ BUILD FAILED"
    echo ""
    echo "🎯 Next Steps:"
    echo "   1. Read the error above carefully"
    echo "   2. Check diagnostic_build.log for full output"
    echo "   3. Common fixes:"
    echo "      - Add missing 'use' statements"
    echo "      - Add missing 'pub mod' declarations"
    echo "      - Check for typos"
    echo ""
    echo "💡 If you see 'cannot find function/value X':"
    echo "   Find where X is defined: grep -rn 'fn X\\|let X\\|const X' src/"
    echo "   Then add proper use/mod statements"
else
    echo "✅ BUILD SUCCESSFUL!"
fi

exit $EXIT_CODE
