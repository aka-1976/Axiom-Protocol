#!/bin/bash
# E0425 Error Analyzer - Identifies "cannot find value/function/module" issues
# Usage: ./e0425_analyzer.sh
# This script analyzes the codebase for common E0425 causes without requiring compilation

set -e

echo "╔════════════════════════════════════════════════════════╗"
echo "║  🔍 E0425 ERROR ANALYZER                              ║"
echo "║  Identifies 'cannot find' errors without compilation   ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

ISSUES_FOUND=0

# ============================================
# Test 1: Verify all module declarations
# ============================================
echo "📋 TEST 1: Module Declarations"
echo "   Checking if all pub mod declarations are present..."
echo ""

REQUIRED_MODULES=(
    "error"
    "config"
    "transaction"
    "block"
    "chain"
    "wallet"
    "vdf"
    "ai_engine"
    "network"
    "storage"
    "mempool"
    "consensus"
    "ai"
    "economics"
    "zk"
    "genesis"
    "bridge"
    "main_helper"
    "time"
    "state"
    "network_config"
    "guardian_sentinel"
    "neural_guardian"
    "openclaw_integration"
    "privacy"
    "sustainability"
    "mobile"
)

while IFS= read -r mod; do
    if [ -z "$mod" ]; then continue; fi
    if ! grep -q "^pub mod $mod;" src/lib.rs; then
        echo "   ❌ Missing: 'pub mod $mod;' in src/lib.rs"
        ((ISSUES_FOUND++))
    else
        echo "   ✅ Found: pub mod $mod"
    fi
done < <(printf '%s\n' "${REQUIRED_MODULES[@]}")

echo ""

# ============================================
# Test 2: Check for missing use statements
# ============================================
echo "📋 TEST 2: Missing Use Statements"
echo "   Checking src/main.rs for potentially missing imports..."
echo ""

# Check if main.rs imports from axiom_core
if grep -q "use axiom_core" src/main.rs; then
    echo "   ✅ src/main.rs imports from axiom_core"
    
    # Extract what it imports
    IMPORTS=$(grep "^use axiom_core:" src/main.rs | head -1)
    echo "   Import statement: $IMPORTS"
else
    echo "   ❌ src/main.rs doesn't import from axiom_core!"
    ((ISSUES_FOUND++))
fi

echo ""

# ============================================
# Test 3: Verify AI module structure
# ============================================
echo "📋 TEST 3: AI Module Structure"
echo "   Checking ai module exists and is properly exported..."
echo ""

if [ ! -d "src/ai" ]; then
    echo "   ❌ ERROR: src/ai directory doesn't exist!"
    ((ISSUES_FOUND++))
else
    echo "   ✅ src/ai directory found"
    
    if [ ! -f "src/ai/mod.rs" ]; then
        echo "   ❌ Missing: src/ai/mod.rs"
        ((ISSUES_FOUND++))
    else
        echo "   ✅ src/ai/mod.rs found"
        
        # Check if oracle is declared
        if grep -q "pub mod oracle" src/ai/mod.rs; then
            echo "   ✅ oracle submodule declared in ai/mod.rs"
        else
            echo "   ❌ Missing: 'pub mod oracle' in src/ai/mod.rs"
            ((ISSUES_FOUND++))
        fi
    fi
    
    if [ ! -f "src/ai/oracle.rs" ]; then
        echo "   ❌ Missing: src/ai/oracle.rs"
        ((ISSUES_FOUND++))
    else
        echo "   ✅ src/ai/oracle.rs found"
    fi
fi

echo ""

# ============================================
# Test 4: Check for undefined types used
# ============================================
echo "📋 TEST 4: Undefined Types in Use Statements"
echo "   Checking for types used but not properly imported..."
echo ""

# Find all use statements in src/
grep -rn "^use [a-zA-Z]" src/ | while read -r line; do
    FILE=$(echo "$line" | cut -d: -f1)
    USE_STMT=$(echo "$line" | cut -d: -f3-)
    
    # Extract the module being imported
    MODULE=$(echo "$USE_STMT" | sed 's/use //g' | cut -d: -f1)
    
    # Check if it's a local module (contains ::)
    if echo "$MODULE" | grep -q "::"; then
        # This is an internal import, check if it references a declared module
        TOP_MODULE=$(echo "$MODULE" | cut -d: -f1)
        
        # Check if the top module is declared in lib.rs
        if ! grep -q "pub mod $TOP_MODULE;" src/lib.rs && ! grep -q "^use $TOP_MODULE" src/lib.rs; then
            if [ "$FILE" != "src/lib.rs" ] && [ "$FILE" != "src/main.rs" ]; then
                echo "   ⚠️  Warning: '$MODULE' used in $FILE but not declared"
            fi
        fi
    fi
done

echo "   ✅ Use statement check complete"
echo ""

# ============================================
# Test 5: Check for missing file declarations
# ============================================
echo "📋 TEST 5: Module Files Don't Exist"
echo "   Checking for declared modules without corresponding files..."
echo ""

grep "^pub mod" src/lib.rs | sed 's/pub mod //g' | sed 's/;//g' | sed 's/ *\/\/.*//' | while read -r mod; do
    if [ -z "$mod" ]; then continue; fi
    
    # Trim whitespace
    mod=$(echo "$mod" | xargs)
    
    # Check if it's a file or directory
    if [ ! -f "src/${mod}.rs" ] && [ ! -d "src/${mod}" ]; then
        echo "   ❌ ERROR: Module 'pub mod $mod;' declared but no src/${mod}.rs or src/${mod}/mod.rs found!"
        ((ISSUES_FOUND++))
    fi
done

echo "   ✅ Module file check complete"
echo ""

# ============================================
# Test 6: Check for duplicate declarations
# ============================================
echo "📋 TEST 6: Duplicate Module Declarations"
echo "   Checking for modules declared twice..."
echo ""

grep "^pub mod" src/lib.rs | sed 's/pub mod //g' | sed 's/;//g' | sed 's/ *\/\/.*//' | sed 's/ //g' | sort | uniq -d | while read -r dup; do
    if [ -z "$dup" ]; then continue; fi
    echo "   ❌ DUPLICATE: pub mod $dup;"
    ((ISSUES_FOUND++))
done

if [ $(grep "^pub mod" src/lib.rs | sed 's/pub mod //g' | sed 's/;//g' | sed 's/ *\/\/.*//' | sed 's/ //g' | sort | uniq -d | wc -l) -eq 0 ]; then
    echo "   ✅ No duplicate module declarations"
fi

echo ""

# ============================================
# Test 7: Check main.rs for undefined uses
# ============================================
echo "📋 TEST 7: main.rs Import Analysis"
echo "   Checking if main.rs imports are all defined modules..."
echo ""

if grep "^use axiom_core::{" src/main.rs > /dev/null; then
    # Extract the modules being imported from the first use axiom_core::{...} line
    MAIN_IMPORTS=$(grep "^use axiom_core::{" src/main.rs | head -1 | sed 's/.*{//g' | sed 's/}.*//' | tr ',' '\n' | xargs -I {} bash -c 'echo "{}" | sed "s/^ *//g" | sed "s/ *$//g"' | sort -u)
    
    echo "   Modules imported by main.rs:"
    while IFS= read -r import; do
        if [ -z "$import" ]; then continue; fi
        
        if grep -q "^pub mod $import" src/lib.rs; then
            echo "   ✅ $import is declared"
        else
            echo "   ❌ $import is NOT declared in lib.rs"
            ((ISSUES_FOUND++))
        fi
    done <<< "$MAIN_IMPORTS"
fi

echo ""

# ============================================
# Summary
# ============================================
echo "╔════════════════════════════════════════════════════════╗"

if [ $ISSUES_FOUND -eq 0 ]; then
    echo "║  ✅ ALL CHECKS PASSED - No obvious E0425 causes found ║"
else
    echo "║  ❌ Found $ISSUES_FOUND potential E0425 issues           ║"
fi

echo "╚════════════════════════════════════════════════════════╝"
echo ""

if [ $ISSUES_FOUND -gt 0 ]; then
    echo "⚠️  Issues found! Review the output above and:"
    echo "   1. Ensure all modules are declared with 'pub mod NAME;' in src/lib.rs"
    echo "   2. Ensure all declared modules have corresponding files/folders"
    echo "   3. Ensure main.rs imports declared modules"
    echo ""
    exit 1
else
    echo "✅ No obvious E0425 issues detected by this analyzer."
    echo "   The E0425 error may be:"
    echo "   1. In a submodule's internal imports"
    echo "   2. A typo in a function/variable name"
    echo "   3. An issue visible only during actual compilation"
    echo ""
    echo "Next steps:"
    echo "   1. Run: cargo check --lib 2>&1 | grep -A 10 'error\\[E0425\\]'"
    echo "   2. Share the full error message from the output above"
    echo ""
    exit 0
fi
