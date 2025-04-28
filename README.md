## This project is now **DEPRECATED**, we are no longer working on it.

# PLEASE, stop saying we are porting PMMP into Rust, that's a bad idea which started as a MEME:
## 1. Different Language Paradigms and Ecosystems:

PHP's Dynamic Nature vs. Rust's Static Typing: PHP is dynamically typed and interpreted, offering rapid development and flexibility. Its ecosystem is vast and specifically tailored for web development, with a plethora of libraries for database interaction, networking, and web frameworks. Rust, on the other hand, is a statically typed, compiled language focused on performance, memory safety, and concurrency. Its ecosystem, while growing rapidly, is different and geared towards systems programming, embedded systems, and high-performance applications. Directly translating PHP's dynamic features and relying on its specific library ecosystem in a statically typed language like Rust would be incredibly complex and likely result in a less idiomatic and potentially less efficient Rust codebase.
Memory Management Differences: PHP uses automatic memory management (garbage collection), which simplifies development but can introduce performance overhead. Rust employs a borrow checker system for memory safety without garbage collection, offering fine-grained control and excellent performance but requiring a different way of thinking about memory management. Porting a large PHP codebase to Rust would necessitate a complete rethinking of memory management strategies, which is a monumental task.
## 2. Ecosystem and Library Mismatch:

PMMP's Reliance on PHP Libraries: PMMP heavily relies on PHP's extensive standard library and its rich ecosystem of packages (e.g., for networking, database interaction, asynchronous operations). Replicating this functionality from scratch in Rust or finding equivalent, mature Rust libraries for every PMMP dependency would be a massive undertaking. Many PHP-specific libraries might not have direct equivalents in the Rust ecosystem.
Community and Expertise: The PMMP community is deeply rooted in PHP. Rewriting it in Rust would mean potentially alienating the existing developer base and requiring a new community with expertise in Rust and the intricacies of the Minecraft: Bedrock Edition server environment to form.
## 3. Performance Considerations are Nuanced:

"Faster" Isn't Always Better in This Context: While Rust generally offers superior raw performance compared to PHP, the performance bottlenecks in a Minecraft server might not solely lie in the language itself. Factors like network I/O, database interactions, game logic efficiency, and the Bedrock Edition's network protocol also play significant roles. Simply rewriting in a "faster" language doesn't guarantee a dramatically better-performing server if these other bottlenecks aren't addressed. Optimizing the existing PHP codebase and architecture might yield more practical performance gains in the short to medium term.
Development Effort vs. Potential Gain: The sheer effort required to rewrite a complex project like PMMP in Rust would be immense. The potential performance gains might not justify the massive investment of time and resources, especially when considering the maturity and existing features of the PHP codebase.
## 4. Architectural Differences:

PMMP's Design in PHP: PMMP's architecture and design patterns are likely influenced by PHP's characteristics. A direct port to Rust might not naturally align with Rust's strengths and could lead to an awkward and unidiomatic Rust codebase that doesn't leverage its concurrency and memory safety features effectively. A better approach would be to design a new system (like Amethyst) from the ground up, taking Rust's capabilities into account.

# PocketMine-RS 🦀🚀

> "Me and my team of highly trained Rustaceans™ have finally done what no sane developer would attempt: rewriting PocketMine-MP in Rust. Why? Because PHP wasn’t giving us enough compile-time existential crises."
> — Rustaceans Incorporated™

# DISCLAIMER:
Everything is from [PocketMine-MP](https://github.com/pmmp/PocketMine-MP), please rate star to the original project and without it we couldn't make this done.

---

## 📋 Table of Contents

1. [About the Project](#about-the-project)  
2. [Why Rust?](#why-rust)  
3. [Vision & Mission](#vision--mission)  
4. [Deep Architectural Dive](#deep-architectural-dive)  
   - [Async Reactor Core](#async-reactor-core)  
   - [Protocol Stack](#protocol-stack)  
   - [Plugin Ecosystem](#plugin-ecosystem)  
   - [Storage & Persistence](#storage--persistence)  
   - [Metrics & Observability](#metrics--observability)  
5. [Core Pillars & Design Principles](#core-pillars--design-principles)  
6. [Team of Professionals](#team-of-professionals)  
7. [What the PMMP Devs Are Saying](#what-the-pmmp-devs-are-saying)  
8. [Getting Started](#getting-started)  
9. [Development Workflow](#development-workflow)  
10. [Testing & Quality Assurance](#testing--quality-assurance)  
11. [Roadmap & Milestones](#roadmap--milestones)  
12. [Community & Support](#community--support)  
13. [FAQ](#faq)  
14. [License & Credits](#license--credits)

---

## About the Project

**PocketMine‑RS** is the most ambitious, unnecessary, and totally epic rewrite of PocketMine‑MP you never asked for. We decided PHP wasn’t giving us enough headaches, so we ported everything into a language where the compiler is smarter than us.

This project is:

- **0% complete**
- **100% serious** (seriously joking)
- **∞% chaos**

The idea came to life during a 3AM voice call when someone said, "wouldn’t it be funny if...?" Spoiler: it was.

---

## Why Rust?

> “Why not write it in JavaScript?” — A banned user

1. **Memory Safety**  
   Rust ensures you can’t shoot yourself in the foot—unless you try really hard. Then it’ll compile and shoot you in the face instead.  
2. **Performance**  
   So fast it outruns your motivation to finish the project.  
3. **Modern Ecosystem**  
   Cargo makes dependency hell a fun little weekend escape.  
4. **Developer Therapy**  
   Nothing beats the emotional rollercoaster of pleasing the borrow checker.

---

## Vision & Mission

- **Vision:** A memory-safe, multithreaded, async-enabled Bedrock server that sometimes boots.  
- **Mission:** Build the most over-engineered Minecraft server of all time and pretend it’s for performance.

---

## Deep Architectural Dive

### Async Reactor Core

- **Custom Event Reactor**: Built entirely on `tokio`, duct tape, and misplaced optimism.  
- **Task Scheduler**: Randomizes which async tasks run first for that authentic chaos vibe.  
- **Hot-Swappable Panic Hooks**: Because you never know when your runtime will just... give up.

### Protocol Stack

- **Bedrock Protocol v1.x-ish**: Inspired by the official protocol, but with more comments that say "???".  
- **Packet Parser**: Validates data like a strict librarian with a taser.  
- **Compression**: Supports GZIP and developer tears.

### Plugin Ecosystem

- **WASM Plugins**: Because nothing screams extensibility like trying to debug a WASM panic in Rust.  
- **Rusty Scripting DSL**: Currently just `println!("Hello plugin world!")`, but dream big.  
- **PHP Bridge**: We sacrificed a goat to make this work. It didn’t.

### Storage & Persistence

- **Chunk Storage**: Implemented using Schrödinger’s serialization—it both works and doesn’t.  
- **Persistence Engine**: Saves data on a quantum level (may or may not be real).  
- **Experimental DB Support**: Now with 97% more SQL injection resistance!

### Metrics & Observability

- **Prometheus**: If it works, it exports. If not, it still exports (errors).  
- **Tracing**: You can trace execution all the way to the root of your poor life choices.  
- **Logs**: Output in JSON, YAML, Morse code, and interpretive dance.

---

## Core Pillars & Design Principles

| Pillar                         | Principle                                                           |
| ------------------------------ | ------------------------------------------------------------------- |
| 🔒 **Safety Last**            | `unsafe` is just another word for spicy.     |
| ⚡ **Speedrun Development**    | Who needs tests when you can YOLO deploy?                  |
| 🧩 **Extensibility++**| Every part is modular, replaceable, and unstable.                   |
| 💡 **Experimental All The Way**| If it compiles, ship it. If it doesn’t, compile harder.    |

---

## Team of Professionals

| Name                | Title                         | Specialty                                            |
| ------------------- | ----------------------------- | ---------------------------------------------------- |
| **Rusty McRustface**    | Lead Compiler Whisperer              | Tames lifetimes with sheer panic         |
| **Ferris the Crab** | Chief Morale Officer          | Snips code and confidence equally              |
| **Borrow Checker**  | Head of HR             | Denies your requests with cryptic notes   |
| **Macrosaurus**    | Code Generator Overlord         | Writes macros that write macros that write errors     |
| **Mr. Segfault**  | QA Director    | Makes sure everything breaks just before release               |

---

## What the PMMP Devs Are Saying

> **@dktapps**  
> "This is either a prank or a cry for help."

> **@shoghicp (Shoghi Cervantes)**  
> "When I said 'rewrite PMMP,' I didn’t mean like this."

> **@intyre**  
> “I read the code. I cried.”

> **PMMP Discord Moderator**  
> "Stop tagging me about this."

---

## Getting Started

1. **Install Rust**  
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **Clone the Repository**  
   ```bash
   git clone https://github.com/sauoro/PocketMine-RS.git
   cd PocketMine-RS
   cargo build --release
   ```
3. **Run the Server**  
   ```bash
   ./target/release/pocketmine-rs --port 19132 --max-players 10
   ```
4. **Regret Everything**  
   Start Minecraft Bedrock. Connect. Watch it maybe work.

---

## Development Workflow

- **Branching Model**:  
  - `main`: For brave souls only.  
  - `dev`: Slightly less cursed.  
  - `panic-hotfix-*`: Used frequently.

- **Code Reviews**:  
  - PRs must include one meme and a bug fix. Preferably unrelated.  

- **CI/CD**:  
  - GitHub Actions, Travis, Jenkins, and probably a hamster wheel.

---

## Testing & Quality Assurance

- **Unit Tests**: Every function tested. Except the broken ones.  
- **Fuzzing**: Because "expected behavior" is just a suggestion.  
- **Benchmarking**: Fast. Probably. We haven't checked.

---

## Roadmap & Milestones

- **v0.0.1-alpha**: Accepts connections and occasionally logs them.  
- **v0.1.0-beta**: Commands maybe work. Don’t quote us.  
- **v1.0.0**: Achieves sentience and forks itself.

---

## Community & Support

- **Discord**: Join for moral support and mutual debugging trauma. [Discord](https://discord.gg/hSTcSRcNcQ) 
- **GitHub Issues**: Please scream responsibly.  

---

## FAQ

**Q: Why does it exist?**  
A: Because no one stopped us in time.

**Q: Can I use it in production?**  
A: You can use _anything_ in production if you're bold enough.

**Q: How stable is it?**  
A: It's stable... in the sense that it fails consistently.

---

## License & Credits

- **License**: MIT — because we legally can’t stop you.  
- **Credits**:  
  - The PMMP devs, for giving us something beautiful to parody  
  - The Rust community, for enabling our hubris  
  - You, for reading this far. You’re the real MVP.

---

_May the borrow checker bless your soul._

shut up
