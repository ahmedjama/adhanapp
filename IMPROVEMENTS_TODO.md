# Improvements TODO

## High Priority (High Impact, Low/Moderate Effort)

1. **Error Handling**
   - Add structured error handling with retries for network requests.
   - Use libraries like `thiserror` or `anyhow` for better error context.
   - Provide more descriptive error messages.

2. **Logging**
   - Replace `println!` with a structured logging library like `log` or `env_logger`.
   - Ensure logs are configurable and provide sufficient detail for debugging.

3. **Resilience to Missing Data**
   - Add fallback mechanisms to estimate prayer times based on previous days or default values.
   - Ensure the app doesn't fail when today's data is missing.

---

## Medium Priority (Moderate Impact, Moderate Effort)

4. **Configuration Management**
   - Cache the configuration in memory after the first read to reduce redundant file I/O.

5. **Thread Management**
   - Implement graceful shutdown for the background thread using `std::sync::mpsc` or `tokio` channels.

6. **Code Organization**
   - Refactor the code into modules for better maintainability:
     - `config.rs` for configuration management.
     - `api.rs` for API calls.
     - `file.rs` for file handling.
     - `main.rs` for the main application logic.

---

## Low Priority (Low Impact, High Effort)

7. **Testing**
   - Add unit tests for critical functions like `fetch_prayer_times_from_file`, `fetch_year_prayer_times_from_api`, and `save_year_prayer_times_to_file`.
   - Add integration tests to ensure end-to-end functionality.

8. **Performance**
   - Cache parsed prayer times in memory and reload the file only if it changes.

9. **Dependency Management**
   - Replace blocking calls with asynchronous versions (e.g., `tokio` with `reqwest`).
   - Refactor the codebase to support async operations.

10. **User Feedback**
    - Add a simple CLI interface or a basic GUI for better user interaction.
    - Provide clear feedback to the user about the app's status and operations.
