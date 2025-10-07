#!/bin/bash
# Parallel per-test-case Bash scanner with detailed logging
# This creates baseline data for each test case subfolder

cd /c/Users/gstra/Code/rust-scanner

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_DIR="analyze/per-testcase-logs/$TIMESTAMP"
mkdir -p "$LOG_DIR"

echo "🚀 Starting parallel per-test-case Bash scans..."
echo "📁 Logs will be in: $LOG_DIR"
echo ""

# Function to run bash scanner on a single test case
run_bash_testcase() {
    local testdir=$1
    local testname=$(basename "$testdir")
    local logfile="$LOG_DIR/bash_${testname}.log"
    
    echo "⏳ [$(date +%H:%M:%S)] Starting: $testname"
    
    # Run bash scanner (normal mode) - use absolute path
    cd shai-hulud-detect
    local abs_testdir=$(realpath "../$testdir")
    timeout 300 ./shai-hulud-detector.sh "$abs_testdir" > "../$logfile" 2>&1
    local exit_code=$?
    cd ..
    
    if [ $exit_code -eq 124 ]; then
        echo "⏱️  [$(date +%H:%M:%S)] TIMEOUT: $testname (>5min)" | tee -a "$logfile"
    elif [ $exit_code -eq 0 ]; then
        echo "✅ [$(date +%H:%M:%S)] Done: $testname" 
    else
        echo "❌ [$(date +%H:%M:%S)] Error: $testname (exit $exit_code)" | tee -a "$logfile"
    fi
    
    # Extract summary
    grep -E "High Risk Issues:|Medium Risk Issues:|Low Risk.*informational" "$logfile" > "$LOG_DIR/bash_${testname}_summary.txt" 2>/dev/null || echo "NO SUMMARY" > "$LOG_DIR/bash_${testname}_summary.txt"
}

# Function to run rust scanner on a single test case
run_rust_testcase() {
    local testdir=$1
    local testname=$(basename "$testdir")
    local logfile="$LOG_DIR/rust_${testname}.log"
    
    echo "⚡ [$(date +%H:%M:%S)] Starting: $testname (Rust)"
    
    # Run rust scanner (normal mode) - use absolute path
    cd dev-rust-scanner-1
    local abs_testdir=$(realpath "../$testdir")
    cargo run --quiet --release -- "$abs_testdir" > "../$logfile" 2>&1
    local exit_code=$?
    cd ..
    
    if [ $exit_code -eq 0 ]; then
        echo "✅ [$(date +%H:%M:%S)] Done: $testname (Rust)"
    else
        echo "❌ [$(date +%H:%M:%S)] Error: $testname (Rust, exit $exit_code)" | tee -a "$logfile"
    fi
    
    # Extract summary
    grep -E "High Risk Issues:|Medium Risk Issues:|Low Risk.*informational" "$logfile" > "$LOG_DIR/rust_${testname}_summary.txt" 2>/dev/null || echo "NO SUMMARY" > "$LOG_DIR/rust_${testname}_summary.txt"
}

export -f run_bash_testcase
export -f run_rust_testcase
export LOG_DIR

# Get all test case directories
TESTCASES=($(find shai-hulud-detect/test-cases -mindepth 1 -maxdepth 1 -type d | sort))

echo "Found ${#TESTCASES[@]} test cases"
echo ""

# Run Bash scans in parallel (max 4 at a time to not overload)
echo "🔵 Phase 1: Running Bash scanners in parallel (max 4 concurrent)..."
printf '%s\n' "${TESTCASES[@]}" | xargs -P 4 -I {} bash -c 'run_bash_testcase "$@"' _ {}

echo ""
echo "🟢 Phase 2: Running Rust scanners in parallel (max 8 concurrent - faster)..."
printf '%s\n' "${TESTCASES[@]}" | xargs -P 8 -I {} bash -c 'run_rust_testcase "$@"' _ {}

echo ""
echo "📊 Creating comparison report..."

# Create comparison CSV
cat > "$LOG_DIR/comparison.csv" << 'CSVHEADER'
TestCase,Bash_High,Bash_Medium,Bash_Low,Rust_High,Rust_Medium,Rust_Low,Match
CSVHEADER

for testdir in "${TESTCASES[@]}"; do
    testname=$(basename "$testdir")
    
    # Extract bash numbers
    bash_high=$(grep "High Risk Issues:" "$LOG_DIR/bash_${testname}_summary.txt" 2>/dev/null | awk '{print $NF}' || echo "0")
    bash_med=$(grep "Medium Risk Issues:" "$LOG_DIR/bash_${testname}_summary.txt" 2>/dev/null | awk '{print $NF}' || echo "0")
    bash_low=$(grep "Low Risk" "$LOG_DIR/bash_${testname}_summary.txt" 2>/dev/null | grep "informational" | awk '{print $NF}' || echo "0")
    
    # Extract rust numbers
    rust_high=$(grep "High Risk Issues:" "$LOG_DIR/rust_${testname}_summary.txt" 2>/dev/null | awk '{print $NF}' || echo "0")
    rust_med=$(grep "Medium Risk Issues:" "$LOG_DIR/rust_${testname}_summary.txt" 2>/dev/null | awk '{print $NF}' || echo "0")
    rust_low=$(grep "Low Risk" "$LOG_DIR/rust_${testname}_summary.txt" 2>/dev/null | grep "informational" | awk '{print $NF}' || echo "0")
    
    # Check match
    if [ "$bash_high" = "$rust_high" ] && [ "$bash_med" = "$rust_med" ] && [ "$bash_low" = "$rust_low" ]; then
        match="✅"
    else
        match="❌"
    fi
    
    echo "$testname,$bash_high,$bash_med,$bash_low,$rust_high,$rust_med,$rust_low,$match" >> "$LOG_DIR/comparison.csv"
done

echo ""
echo "✅ Done! Results in: $LOG_DIR"
echo ""
echo "📄 Comparison CSV:"
cat "$LOG_DIR/comparison.csv"

# Summary
total_tests=${#TESTCASES[@]}
matched=$(grep "✅" "$LOG_DIR/comparison.csv" | wc -l)
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 SUMMARY: $matched / $total_tests test cases match perfectly"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
