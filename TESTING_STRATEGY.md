# Testing Strategy & Scenario Identification Guide

This document provides a systematic approach to identifying test scenarios for the `fos_server` project. It helps in moving beyond "Happy Path" testing to cover edge cases, user errors, and architectural robustness.

## 1. The State Transition Matrix (Core Logic)

The most effective way to identify logic bugs in a state-machine-driven application (like Bevy) is a Transition Matrix.

### How to Create It
1.  **Rows (Current State):** List all possible states a system can be in (e.g., `Starting`, `Running`, `Stopping`).
2.  **Columns (Events/Triggers):** List all external events that can occur (e.g., `NetMessage`, `UserClick`, `Timeout`).
3.  **Cells:** Mark each intersection with:
    *   ✅ **Happy Path:** Expected behavior.
    *   ⚠️ **Edge Case:** Possible but tricky (needs handling).
    *   ❌ **Impossible/Bug:** Should strictly not happen (architectural invariant).

### Example: Singleplayer Lifecycle

| Current State | Event: `SetupDone` | Event: `Exit (ESC)` | Event: `Disconnect` |
| :--- | :--- | :--- | :--- |
| **Starting** | ✅ -> Running | ⚠️ **Abort Mid-Load** | ❌ Irrelevant |
| **Running** | ❌ (Bug) | ✅ -> Stopping | ✅ -> Stopping |
| **Stopping** | ❌ (Bug) | ⚠️ Double Stop? | ✅ Ignore |
| **Menu** | ❌ (Bug) | ❌ Ignore | ❌ Irrelevant |

**Checklist Derivation:**
*   Do I have a test for `Starting` + `Exit (ESC)`? -> *This checks if we crash when aborting loading.*
*   Do I have a test for `Stopping` + `Exit (ESC)`? -> *This checks if spamming ESC causes double-free errors.*

## 2. The Lifecycle Method (CRUD for Games)

Every feature follows a lifecycle. Check "Interruptions" at every stage.

*   **C - Create (Spawn/Load):**
    *   *Scenario:* Abort immediately after start.
    *   *Scenario:* Trigger start twice rapidly (Double-Click).
    *   *Scenario:* Start fails (e.g., port blocked).
*   **R - Read (Run/Update):**
    *   *Scenario:* Simulation runs while window is minimized/unfocused.
    *   *Scenario:* Simulation pauses correctly when menu opens.
*   **U - Update (Change State):**
    *   *Scenario:* Changing level/settings while game is running.
*   **D - Delete (Despawn/Cleanup):**
    *   *Scenario:* Restart immediately after stop (Resource leaks?).
    *   *Scenario:* Quit application while in-game (vs. going to menu first).

## 3. The "3 Vs" of Input (User Error)

When simulating user input, consider these three dimensions:

1.  **Velocity (Speed):**
    *   *Test:* Pressing buttons faster than the framerate.
    *   *Test:* Opening/Closing menus in 1 frame.
    *   *Goal:* Detect Race Conditions (e.g., `unwrap` on an entity that is already despawned).
2.  **Volume (Quantity):**
    *   *Test:* No input vs. Massive input.
    *   *Test:* Empty strings, extremely long strings.
    *   *Goal:* Buffer overflows, UI layout breaks.
3.  **Validity (Context):**
    *   *Test:* Sending "Shoot" command while in "Inventory".
    *   *Test:* Sending "Join Game" while already in a game.
    *   *Goal:* Verify state guards (`.run_if(in_state(...))`).

## 4. Practical Checklist Generation

Combine the above into a concrete checklist for your feature.

**Feature: Singleplayer Mode**

**Tier 1: Happy Path (Must Have)**
- [ ] Start Game -> Play -> Stop Game -> Menu (Verifies basic flow).
- [ ] Save Game -> Load Game (Verifies persistence).

**Tier 2: Edge Cases (Robustness)**
- [ ] **Mid-Start Abort:** Press Exit while `Starting`. (Matrix: `Starting` + `Exit`).
- [ ] **Rapid Restart:** Start -> Stop -> Start immediately. (Lifecycle: `Delete` -> `Create`).
- [ ] **Menu Toggle Spam:** Open/Close menu 10 times in 1 second. (3Vs: Velocity).

**Tier 3: Developer/Architectural Constraints**
- [ ] Verify that systems for `InGame` do NOT run when in `Menu`.
- [ ] Verify that `LocalServer` entity is strictly unique (no duplicates).

## Summary

1.  **Draw the Matrix** for your states.
2.  **Interrupt the Lifecycle** (Stop during Start).
3.  **Stress the Inputs** (Fast, Many, Wrong Context).
4.  **Write Tests** for the red flags (⚠️) identified.
