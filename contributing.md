# Contributing to Rust lms client 🦀

Hey there! 👋 We're thrilled that you're interested in contributing to our project! Whether you're a seasoned Rustacean or just getting started, your input is valuable, and we're here to help you every step of the way.

## How to Get Started 🚀

1. **Fork the Repo**: Start by forking the repository. Click the "Fork" button at the top right of this page to create a copy of the repo under your own GitHub account.

2. **Clone Your Fork**: Clone your forked repository to your local machine. You can do this using the command line:
   ```sh
   git clone https://github.com/YOUR-USERNAME/YOUR-FORKED-REPO.git
   cd YOUR-FORKED-REPO
   ```

3. **Install rust** Before you start working on the project, make sure you have Rust installed on your system.
If you haven't already, you can install Rust using the following command:
    ```sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
Follow the instructions in the terminal to complete the installation. After installing, run:
    ```sh
        rustup update
    ```

4. Start Contributing! 🎉

Now that you have Rust installed and the repository set up locally, you’re ready to start contributing! Here are a few steps to guide you:

- **Create a new branch**: Always create a new branch for your changes to keep your `main` branch clean.
  ```sh
  git checkout -b your-feature-branch
  ```
- Make your changes: Whether you're fixing a bug, adding a new feature, or improving the documentation, make your changes in your new branch.

- Compile the project: Ensure your changes compile successfully by running:
    ```sh
    cargo build
    ```

- Run tests: Before submitting, run tests to make sure your changes don't break anything:
    ```sh
    cargo test
    ```
    
5. Commit and Push Your Changes 📤
- Commit your changes: Make sure your commit messages are descriptive, so others know what you've done.
    ```sh
    git add .
    git commit -m "Add detailed description of your changes"
    ```
- Push to your fork: Push your changes to your forked repository on GitHub:
    ```sh
    git push origin your-feature-branch
    ```

6. Open a Pull Request (PR) 🔄
Once your changes are pushed to your fork:

1. Go to the original repository on GitHub.
2. Click the Pull Requests tab.
3. Click the New Pull Request button.
4. Choose your branch from your fork to merge into the main branch of the original repository.
5. Provide a clear and detailed description of your changes and why they're needed.
6. Submit the Pull Request!

7. Respond to Feedback 🗣️
A maintainer will review your pull request and may provide feedback. Please be patient, respond to comments, and make any requested changes. Remember, we’re all here to build something great together!

Thank You for Contributing! 🙏
We appreciate every bit of help we can get, and we're excited to see what you’ll contribute! Let's build something awesome together. 🚀
