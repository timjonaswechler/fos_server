# Testing Strategy & Scenario Identification Guide

This document is a comprehensive guide to systematically identifying test scenarios for the `fos_server` project. It bridges the gap between raw code and effective test cases by providing methods to discover "What could go wrong?".

## 1. The State Transition Matrix (The Logic Map)

The goal of this matrix is to find out what happens when an event occurs at the "wrong" time.

### Step 1: Find the Rows (The "Where am I?")
Your rows are the stable phases your game can be in. In Bevy, these are usually your States.
*   **Look at your Code:** Search for `#[derive(States)]` or your `states.rs` file.
*   **Example Rows:** `Starting`, `Running`, `Stopping`, `InMenu`.

### Step 2: Find the Columns (The "What happens?")
This is the hardest part. Use the **3 Sources of Disturbance** to find your columns:

1.  **Source 1: The User (Inputs)**
    *   *Method:* Look at your UI. Every button that changes a screen is a column.
    *   *Examples:* Pressing `ESC`, Clicking `Exit`, Clicking `Start Game`, `Save`, `Load`.
2.  **Source 2: The System (Internal Logic)**
    *   *Method:* Look for `commands.trigger(...)` in your code. These are internal signals.
    *   *Examples:* `SetupComplete`, `LoadingFinished`, `GameWon`, `GameLost`.
3.  **Source 3: The Environment (External Factors)**
    *   *Method:* Think about "Murphy's Law". What can break outside your code?
    *   *Examples:* Network Disconnect, Window Closed (ALT+F4), Save File Corrupted.

### Step 3: Fill the Intersection Cells
Ask: *"If I am in [Row State], and [Column Event] happens, what does the code do vs. what SHOULD it do?"*

| State (Row) | Event: User presses ESC (Column) | Event: Loading Finished (Column) |
| :--- | :--- | :--- |
| **Starting** | ⚠️ **Risk:** Abort loading mid-way. Are entities half-spawned? | ✅ **Happy Path:** Transition to Running. |
| **Running** | ✅ **Happy Path:** Open Pause Menu. | ❌ **Bug:** Logic error (why load again?). |
| **Stopping** | ⚠️ **Risk:** Double-free? (User spamming Exit). | ❌ **Bug:** Too late. |

---

## 2. The Lifecycle Method (CRUD for Games)

Every feature in your game (Singleplayer, Multiplayer, Inventory, etc.) is an object that lives and dies. Use the **CRUD** model to find holes in this life.

### How to apply it:
Pick a feature (e.g., "Singleplayer Session") and ask these specific questions:

*   **C - Create (Start/Load)**
    *   *Interruption:* Can I stop it while it's creating? (e.g., Cancel button during loading).
    *   *Repetition:* What if I trigger "Create" twice instantly? (Double-click Start).
    *   *Prerequisites:* What if I try to Create without requirements? (Start game with no map selected).
*   **R - Read (Run/Play)**
    *   *Focus:* Does it keep running if I alt-tab? Should it?
    *   *Concurrency:* Can two sessions run at once? (Should be impossible).
*   **U - Update (Change Settings)**
    *   *Runtime Changes:* Can I change difficulty/resolution while playing? Does it crash?
*   **D - Delete (Stop/Quit)**
    *   *Completeness:* Does it clean up EVERYTHING? (Check entity counts).
    *   *Restart:* Can I immediately Create again after Delete? (The "Resource Leak" check).

---

## 3. The "3 Vs" of Input (Stress Testing)

When you test user input (Source 1 from the Matrix), use these 3 dimensions to find edge cases.

### 1. Velocity (Speed)
*   *The Question:* "What if the user is faster than the game logic?"
*   *Scenarios:*
    *   Clicking "Next" -> "Back" -> "Next" in 100ms.
    *   Pressing "Exit" immediately after "Start".
    *   *Why?* Detects Race Conditions (e.g., trying to despawn an entity that hasn't finished spawning).

### 2. Volume (Quantity)
*   *The Question:* "What if the input is too much or too little?"
*   *Scenarios:*
    *   Entering a 0-character name.
    *   Entering a 10,000-character name.
    *   Spamming the "Fire" button 50 times/sec.
    *   *Why?* Detects Buffer Overflows, UI Layout breaks, Performance bottlenecks.

### 3. Validity (Context)
*   *The Question:* "What if the input makes no sense right now?"
*   *Scenarios:*
    *   Sending a "Move Character" command while in the "Main Menu".
    *   Trying to "Join Game" when already connected.
    *   *Why?* Verifies your State Guards (e.g., `.run_if(in_state(InGame))`).

---

## 4. How to Build Your Checklist (Summary)

To create a checklist for a new feature:

1.  **List the States.** (Rows)
2.  **List the Inputs.** (Columns - Buttons, Events).
3.  **Check intersections** for "Happy Path" (✅) vs "Danger Zone" (⚠️).
4.  **Apply Lifecycle Questions** (Can I interrupt Create? Can I restart after Delete?).
5.  **Apply 3 Vs** (Fast clicks? Bad data?).

### Example Checklist for `Singleplayer`:

*   [ ] **Matrix:** `Starting` + `Loading Finished` -> Game Runs (Happy Path).
*   [ ] **Matrix/Velocity:** `Starting` + `Exit Button` -> Clean Abort (Edge Case).
*   [ ] **Lifecycle (D):** `Stopping` -> `Starting` immediately -> No errors (Restart Robustness).
*   [ ] **Validity:** Trigger `NavigateGameMenu` event while in `MainMenu` -> Should be ignored.