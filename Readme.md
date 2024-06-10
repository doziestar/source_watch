# SourceWatch

SourceWatch is a powerful and robust Rust application designed for managing and graphically viewing various data sources. It supports SQL databases, MongoDB, Redis, and log data sources, providing a unified interface to query and visualize data.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

## Features

- **Multi-Database Support:** Connect to and manage SQL databases (MySQL, PostgreSQL), MongoDB, and Redis.
- **Log Data Sources:** Integrate and visualize log data sources.
- **Graphical User Interface:** Intuitive GUI built with `iced` for easy interaction and data visualization.
- **Asynchronous Operations:** Efficiently handle multiple database connections and operations using `tokio` and `async-std`.
- **Robust Logging:** Centralized logging for monitoring and debugging.
- **Extensibility:** Modular architecture for easy extension and maintenance.

## Installation

### Prerequisites

- Rust (latest stable version)
- Cargo (package manager for Rust)

### Steps

1. **Clone the Repository**

   ```bash
   git clone https://github.com/doziestar/source_watch.git
   cd source_watch
    ```
2. **Build the Application**

   ```bash
   cargo build --release
   ```
   
3. **Run the Application**

   ```bash
    cargo run --release
    ```

## Usage

1. **Connect to a Database**

   - Click on the `+` button in the sidebar to add a new database connection.
   - Select the database type (SQL, MongoDB, Redis).
   - Enter the connection details (host, port, username, password).
   - Click `Connect` to establish the connection.

2. **Query Data**

   - Select the database connection from the sidebar.
   - Enter an SQL query or command in the query editor.
   - Click `Run` to execute the query and view the results.
   - The results will be displayed in a table format.
   - Click on the `Chart` button to visualize the data in a graphical format.
   - Save the query results as a CSV file by clicking on the `Save` button.
   - Clear the query editor by clicking on the `Clear` button.
   - Disconnect from the database by clicking on the `Disconnect` button.
   - Close the database connection by clicking on the `X` button.
   - Refresh the database connection by clicking on the `Refresh` button.
   - Edit the database connection details by clicking on the `Edit` button.
   - Delete the database connection by clicking on the `Delete` button.
   - Export the database connection details as a JSON file by clicking on the `Export` button.
   - Import a database connection from a JSON file by clicking on the `Import` button.
   - Search for a specific table or column by entering the search term in the search bar.
   - Filter the query results by entering a filter condition in the filter bar.
   - Sort the query results by clicking on the column headers.
   - Resize the columns by dragging the column dividers.
   - Copy the query results to the clipboard by clicking on the `Copy` button.
   - Export the query results as a CSV file by clicking on the `Export` button.
   - Export the query results as a JSON file by clicking on the `Export` button.

3. **Visualize Data**

   - Click on the `Chart` button to visualize the query results in a graphical format.
   - Select the chart type (line, bar, pie, scatter, etc.).
   - Customize the chart settings (title, labels, colors, etc.).
   - Save the chart as an image file by clicking on the `Save` button.
   - Export the chart data as a CSV file by clicking on the `Export` button.
   - Export the chart data as a JSON file by clicking on the `Export` button.

4. **Manage Log Data**

    - Click on the `Logs` button in the sidebar to view log data sources.
    - Add a new log data source by clicking on the `+` button.
    - Select the log file or directory to monitor.
    - Configure the log data source settings (format, encoding, interval, etc.).
    - Start monitoring the log data source by clicking on the `Start` button.
    - Stop monitoring the log data source by clicking on the `Stop` button.
    - View the log data in real-time as it is generated.
    - Search for specific log entries by entering the search term in the search bar.
    - Filter the log entries by entering a filter condition in the filter bar.
    - Export the log entries as a CSV file by clicking on the `Export` button.
    - Export the log entries as a JSON file by clicking on the `Export` button.

## Configuration

SourceWatch can be configured using a `config.toml` file located in the project root directory. The configuration file allows you to customize various settings such as the application theme, database connections, log data sources, etc.

Here is an example configuration file:

```toml
[databases]
[[databases.mysql]]
name = "MySQL Database"
host = "localhost"
port = 330
username = "root"
password = "password"
database = "test"

[[databases.postgresql]]
name = "PostgreSQL Database"
host = "localhost"
port = 543
username = "postgres"
password = "password"
database = "test"

[[databases.mongodb]]
name = "MongoDB Database"
host = "localhost"
port = 270
username = "admin"
password = "password"
database = "test"

[[databases.redis]]
name = "Redis Database"
host = "localhost"
port = 637
password = "password"

[logs]
[[logs.file]]
name = "Log File"
path = "/var/log/app.log"
format = "json"
encoding = "utf-8"
interval = 1
```

## Development

To contribute to SourceWatch, follow these steps:

1. **Fork the Repository**

   Click on the `Fork` button in the top right corner of the repository page.
2. **Clone the Repository**

   ```bash
   git clone https://github.com/doziestar/source_watch.git
    cd source_watch
    ```
   
3. **Create a New Branch**

   ```bash
   git checkout -b feature/my-feature
    ```
   
4. **Make Changes**

   Make your desired changes to the codebase.
5. **Commit Changes**

   ```bash
    git commit -am "Add new feature"
     ```
   
6. **Push Changes**
7. **Create a Pull Request**

   Click on the `Pull Request` button on the repository page.
8. **Review Changes**

   Wait for the maintainers to review your changes.
9. **Merge Pull Request**

   Once approved, your changes will be merged into the main branch.
10. **Celebrate**

   Congratulations! You have successfully contributed to SourceWatch.

## Contributing

If you would like to contribute to SourceWatch, please read the [CONTRIBUTING.md](CONTRIBUTING.md) file for guidelines and best practices.

## License

SourceWatch is licensed under the [MIT License](LICENSE).
```