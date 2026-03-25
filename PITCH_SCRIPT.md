### 🎬 OneSentinel: Autonomous AI Trading Agent (OneHack 3.0 Pitch)

**(Start Screen Recording: Show your IDE with the terminal open)**

**[0:00 - Introduction]**
"Hello OneHack judges! I’m here to present **OneSentinel** — a fully autonomous, headless AI trading agent built natively for OneChain in Rust. 

Most 'AI crypto bots' out there just hook up an LLM to a web-wallet popup. OneSentinel is completely different. It operates autonomously on a backend daemon. It analyzes live momentum, enforces risk management thresholds, and constructs Native Programmable Transaction Blocks directly on the SUI-compatible OneChain protocol."

**(Type `cargo run --bin demo` in the terminal and hit Enter)**

**[0:30 - Wallet & Security]**
"When I run the initialization, you’ll notice the very first thing it does is boot up its own cryptographic wallet. 

We used `fastcrypto` to derive an `Ed25519` keypair strictly adhering to the `m/44'/784'/0'/0'/0'` standard derivation path. There are no browser extensions here; the bot holds its own keys and signs its own intents programmatically. 

I've already funded this testnet address via the OneLabs Faucet, so I'll hit enter to let the bot continue."

**(Hit Enter in the terminal to proceed past the funding pause)**

**[1:00 - The AI Brain & Risk Manager]**
"Now, the bot is waking up its multi-agent LLM pipeline. 

Instead of simple if/else logic, OneSentinel feeds real-world market metrics into a cutting-edge LLM. In the terminal, you can see it evaluating a mocked 'TEST_TOKEN'. It digests price momentum, volatility, and volume trends to produce a structured psychological thesis: *'Bullish momentum with strong volume confirmation...'* 

It doesn't just decide to 'Buy' — the internal Risk Manager agent steps in and autonomously caps the execution size to `0.45 OCT` to preserve capital."

**[1:45 - PTB Execution & Cryptography]**
"Once the AI signs off on the trade, the Execution Engine takes over. 

Instead of a simple coin transfer, the bot constructs a real **Programmable Transaction Block (PTB)** entirely in Rust. It wraps this inside an `IntentMessage`, manually hashes it with `Blake2b256`, and mathematically applies its Ed25519 signature."

**(Highlight the SUCCESS terminal output and the Transaction Digest)**

**[2:15 - The Proof]**
"And here is the proof of execution. The network validators successfully verified the bot's mathematical signature. 

For this demo, because the official Mainnet DeepBook contracts aren't deployed yet, we submitted an *Empty PTB*. But what this proves is that the hardest part—the cryptography, the AI reasoning, and the autonomous network broadcast—is 100% operational on the OneChain testnet right now."

**[2:40 - Mainnet Readiness & Conclusion]**
"When OneDEX goes live on mainnet, we simply uncomment three lines of code. It will instantly begin executing Native DeepBook `clob_v2` swaps within this exact same verified execution loop. 

OneSentinel is completely headless, deeply integrated with OneChain's Rust SDK, and ready to dominate the DeFi ecosystem without human intervention. Thank you!"

***

**💡 Pro-Tip for the Recording:**
Keep the terminal text large enough to read, and highlight the text with your cursor as you talk about it (like highlighting the `SuiAddress` when you mention the wallet, and highlighting the `Transaction Digest` when you finish).
