# AI Agent Development Guide

This guide provides comprehensive instructions for AI agents working on the iTrader Backend project.

## 🎯 Core Principles

1. **Test-Driven Development (TDD)** - Write tests first, then code
2. **Documentation First** - Document what you're building before you build it
3. **Safety First** - Always run safe tests before dangerous ones
4. **Clear Communication** - Every function must be documented
5. **Continuous Verification** - Test after every change

## 📝 Documentation Standards

### Function Documentation

Every function MUST have comprehensive documentation:

```rust
/// Brief one-line description of what this function does.
///
/// # Description
/// Detailed explanation of the function's purpose, behavior, and any important
/// implementation details that future developers should know.
///
/// # Arguments
/// * `param1` - Description of first parameter and its valid values
/// * `param2` - Description of second parameter and constraints
///
/// # Returns
/// Description of what the function returns and under what conditions
///
/// # Errors
/// * `ErrorType1` - When this error occurs and why
/// * `ErrorType2` - Another possible error condition
///
/// # Examples
/// ```
/// let result = function_name("value1", 42)?;
/// assert_eq!(result, expected_value);
/// ```
///
/// # Panics
/// Conditions under which this function will panic (if any)
///
/// # Safety
/// Any safety considerations when using this function
pub async fn function_name(param1: &str, param2: u32) -> Result<ReturnType> {
    // Implementation
}
```

### Module Documentation

Each module must start with:

```rust
//! Brief description of this module's purpose.
//!
//! # Overview
//! Detailed explanation of what this module provides and how it fits
//! into the overall system architecture.
//!
//! # Usage
//! ```
//! use crate::module_name::{Type1, Type2};
//! 
//! let instance = Type1::new();
//! ```
//!
//! # Features
//! - Feature 1: Description
//! - Feature 2: Description
```

### Type Documentation

```rust
/// Represents a [brief description].
///
/// # Fields
/// * `field1` - Purpose and constraints
/// * `field2` - Purpose and valid values
///
/// # Invariants
/// - List any invariants that must be maintained
#[derive(Debug, Clone)]
pub struct TypeName {
    /// Documentation for this specific field
    pub field1: String,
    
    /// Documentation for private fields too
    field2: u32,
}
```

## 🧪 Testing Guidelines

### Test Categories

1. **SAFE Tests** (Read-only operations)
   - Authentication checks
   - Data retrieval
   - Parsing operations
   - Mock simulations

2. **DANGEROUS Tests** (Modify data)
   - Login operations
   - Transaction approvals
   - Balance modifications
   - Ad creation/deletion

### Running Tests

```bash
# Run all safe tests only
./test.sh all --safe

# Run specific test
./test.sh gate-auth

# List all available tests
./test.sh list

# Run with verbose output
./test.sh all --safe --verbose
```

### Writing Tests

#### Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test [specific functionality being tested]
    ///
    /// This test verifies that [expected behavior]
    #[tokio::test]
    async fn test_descriptive_name() {
        // Arrange - Set up test data
        let input = prepare_test_data();
        let expected = ExpectedResult::new();
        
        // Act - Execute the function
        let result = function_under_test(input).await;
        
        // Assert - Verify the outcome
        assert!(result.is_ok(), "Function should succeed");
        assert_eq!(result.unwrap(), expected, "Result should match expected");
        
        // Additional assertions for side effects
        verify_side_effects();
    }
    
    /// Test error handling for [error condition]
    #[tokio::test]
    async fn test_error_condition() {
        // Arrange
        let invalid_input = create_invalid_input();
        
        // Act
        let result = function_under_test(invalid_input).await;
        
        // Assert
        assert!(result.is_err(), "Should return error");
        match result.unwrap_err() {
            Error::ExpectedError(msg) => {
                assert!(msg.contains("expected text"));
            }
            _ => panic!("Wrong error type"),
        }
    }
}
```

#### Integration Test Template

```rust
/// Integration test for [workflow/feature]
///
/// Tests the complete flow from [start] to [end]
#[tokio::test]
#[ignore] // Remove if test is safe
async fn test_integration_workflow() {
    // Setup
    let test_env = setup_test_environment().await;
    
    // Step 1: [First operation]
    info!("Testing step 1: [description]");
    let step1_result = perform_step1(&test_env).await?;
    assert_step1_success(&step1_result);
    
    // Step 2: [Second operation]
    info!("Testing step 2: [description]");
    let step2_result = perform_step2(&step1_result).await?;
    assert_step2_success(&step2_result);
    
    // Cleanup
    cleanup_test_environment(test_env).await;
}
```

## 🏗️ Code Structure Guidelines

### File Organization

```
src/
├── module_name/
│   ├── mod.rs          # Module declaration and exports
│   ├── models.rs       # Data structures
│   ├── client.rs       # External service client
│   ├── service.rs      # Business logic
│   └── tests.rs        # Unit tests (or in each file)
```

### Error Handling

```rust
use thiserror::Error;

/// Module-specific errors with clear descriptions
#[derive(Error, Debug)]
pub enum ModuleError {
    /// Occurs when [condition]
    #[error("Failed to process {item}: {reason}")]
    ProcessingError { item: String, reason: String },
    
    /// Network-related errors
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    /// Invalid input provided
    #[error("Invalid {field}: {message}")]
    ValidationError { field: String, message: String },
}

// Always provide context
pub fn process_item(item: &str) -> Result<ProcessedItem> {
    validate_item(item)
        .map_err(|e| ModuleError::ValidationError {
            field: "item".to_string(),
            message: e.to_string(),
        })?;
    
    perform_processing(item)
        .map_err(|e| ModuleError::ProcessingError {
            item: item.to_string(),
            reason: e.to_string(),
        })
}
```

### Logging Standards

```rust
use tracing::{info, debug, warn, error};

// Function entry/exit for important operations
pub async fn important_operation(id: &str) -> Result<()> {
    info!("Starting operation for id: {}", id);
    debug!("Operation parameters: {:?}", get_params());
    
    match perform_work().await {
        Ok(result) => {
            info!("Operation completed successfully: {}", id);
            debug!("Result details: {:?}", result);
            Ok(())
        }
        Err(e) => {
            error!("Operation failed for {}: {}", id, e);
            Err(e)
        }
    }
}

// Use structured logging
info!(
    transaction_id = %tx.id,
    amount = %tx.amount,
    status = %tx.status,
    "Processing transaction"
);
```

## 🔄 Development Workflow

### 1. Understanding the Task

Before coding:
1. Read existing code in the module
2. Check for similar implementations
3. Review related tests
4. Understand the data flow

### 2. Planning

Create a plan:
```markdown
## Task: [Feature Name]

### Objective
[What needs to be accomplished]

### Current State
[How it works now]

### Desired State
[How it should work]

### Changes Required
1. [ ] Change 1
2. [ ] Change 2
3. [ ] Tests needed

### Risk Assessment
- **Safe Changes**: [List]
- **Dangerous Changes**: [List]
```

### 3. Implementation Steps

1. **Write failing tests first**
   ```bash
   # Create test file
   touch tests/feature_test.rs
   
   # Run to see it fail
   cargo test feature_test
   ```

2. **Implement minimal code**
   - Start with simplest implementation
   - Make the test pass
   - Don't over-engineer

3. **Refactor**
   - Improve code quality
   - Add error handling
   - Optimize if needed

4. **Document thoroughly**
   - Add function docs
   - Update module docs
   - Add usage examples

5. **Test comprehensively**
   ```bash
   # Run specific tests
   cargo test feature_name
   
   # Run all module tests
   cargo test module_name::
   
   # Run all safe tests
   ./test.sh safe
   ```

### 4. Common Patterns

#### Async Service Pattern

```rust
pub struct ServiceName {
    client: Arc<Client>,
    config: ServiceConfig,
}

impl ServiceName {
    /// Creates a new service instance
    pub fn new(client: Arc<Client>, config: ServiceConfig) -> Self {
        Self { client, config }
    }
    
    /// Performs the main service operation
    pub async fn perform_operation(&self, input: Input) -> Result<Output> {
        // Validate input
        self.validate_input(&input)?;
        
        // Log operation start
        info!("Starting operation with input: {:?}", input);
        
        // Perform work with retries
        let result = retry_with_backoff(|| async {
            self.client.call_api(&input).await
        }).await?;
        
        // Process result
        let output = self.process_result(result)?;
        
        // Log success
        info!("Operation completed successfully");
        
        Ok(output)
    }
}
```

#### Repository Pattern

```rust
#[async_trait]
pub trait Repository<T> {
    async fn find_by_id(&self, id: &str) -> Result<Option<T>>;
    async fn save(&self, entity: &T) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn list(&self, filter: Filter) -> Result<Vec<T>>;
}
```

## 📊 Monitoring Transaction Processing

The orchestrator checks for pending transactions every 5 minutes:

```rust
// Configuration
const CHECK_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes

// Monitoring pattern
loop {
    // Check for new transactions
    let pending = gate_api.get_pending_transactions().await?;
    
    if !pending.is_empty() {
        info!("Found {} pending transactions", pending.len());
        
        for tx in pending {
            // Process immediately
            orchestrator.process_transaction(tx).await?;
        }
    }
    
    // Wait for next check
    tokio::time::sleep(CHECK_INTERVAL).await;
}
```

## 🚨 Safety Checklist

Before committing:

- [ ] All tests pass: `./test.sh all`
- [ ] No compiler warnings: `cargo check`
- [ ] Code is formatted: `cargo fmt`
- [ ] Lints pass: `cargo clippy`
- [ ] Documentation complete
- [ ] Safe tests run successfully
- [ ] Changes are backward compatible
- [ ] Error messages are helpful
- [ ] Logging is appropriate
- [ ] No hardcoded secrets

## 📚 Additional Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Error Handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Async Programming](https://rust-lang.github.io/async-book/)

## 🤝 Collaboration Tips

1. **Ask Questions**: If unclear, ask for clarification
2. **Show Progress**: Share intermediate results
3. **Test Often**: Run tests after each change
4. **Document Assumptions**: Note any assumptions made
5. **Propose Improvements**: Suggest better approaches

Remember: Good code is code that others can understand and maintain!