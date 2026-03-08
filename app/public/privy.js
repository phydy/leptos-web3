/**
 * privy-bridge.js
 *
 * Implements SIWE (MetaMask) and SIWS (Phantom) login against Privy's auth
 * REST API — https://auth.privy.io/api/v1/siwe/*
 *
 * Why REST API instead of js-sdk-core methods?
 *   js-sdk-core only exposes privy.auth.{email,sms,oauth} for embedded flows.
 *   There is NO privy.auth.siwe or privy.auth.siws namespace in the package.
 *   External wallet authentication must go through the REST API directly.
 *
 * Install:
 *   npm install @privy-io/js-sdk-core
 *
 * Bundle (run once, or let Trunk's pre_build hook do it):
 *   npx esbuild js-src/privy-bridge.js \
 *     --bundle --format=esm --outfile=privy-bridge.js
 */

import Privy, {
    LocalStorage,
    getEntropyDetailsFromAccount,
} from "@privy-io/js-sdk-core";
import {
    Connection, PublicKey, Transaction, SystemProgram, LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
    getAssociatedTokenAddress, createTransferInstruction,
} from "@solana/spl-token";

// ---------------------------------------------------------------------------
// Module-level state
// ---------------------------------------------------------------------------
/** @type {InstanceType<typeof Privy> | null} */
let privy = null;
let _appId = "";
let _session = null; // { token, refresh_token, user }
let _iframeReady = false;

const PRIVY_AUTH_URL = "https://auth.privy.io/api/v1";
const SESSION_KEY = "privy_bridge_session";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
function assertReady() {
    if (!_appId) throw new Error("Bridge not initialised — call init() first");
}

/** Common headers required on every Privy auth REST call */
function authHeaders() {
    return {
        "Content-Type": "application/json",
        "privy-app-id": _appId,
    };
}

/**
 * POST to a Privy auth endpoint, throw on non-2xx.
 * @param {string} path  — e.g. "/siwe/init"
 * @param {object} body
 */
async function privyPost(path, body) {
    const res = await fetch(`${PRIVY_AUTH_URL}${path}`, {
        method: "POST",
        headers: authHeaders(),
        body: JSON.stringify(body),
    });

    const json = await res.json().catch(() => ({}));

    if (!res.ok) {
        const msg = json?.message ?? json?.error ?? `HTTP ${res.status}`;
        throw new Error(`Privy API error (${path}): ${msg}`);
    }

    return json;
}

/** Normalise the user object returned by either REST or sdk-core */
function serializeUser(user) {
    if (!user) return null;

    // REST API returns linked_accounts (snake_case); sdk-core returns linkedAccounts
    const linked = user.linked_accounts ?? user.linkedAccounts ?? [];

    const wallet =
        linked.find((a) => a.type === "wallet" && a.chain_type === "ethereum") ??
        linked.find((a) => a.type === "wallet" && a.chain_type === "solana") ??
        null;

    return {
        id: user.id,
        linkedAccounts: linked,
        wallet: wallet
            ? {
                address: wallet.address,
                chainType: wallet.chain_type,
                walletClient: wallet.wallet_client_type ?? wallet.connector_type ?? null,
            }
            : null,
        createdAt: user.created_at ?? user.createdAt ?? null,
    };
}

/** Persist session to localStorage so page reloads restore auth state */
function saveSession(session) {
    _session = session;
    try { localStorage.setItem(SESSION_KEY, JSON.stringify(session)); } catch (_) { }
}

function clearSession() {
    _session = null;
    try { localStorage.removeItem(SESSION_KEY); } catch (_) { }
}

function loadSession() {
    try {
        const raw = localStorage.getItem(SESSION_KEY);
        if (raw) _session = JSON.parse(raw);
    } catch (_) { }
}

// ---------------------------------------------------------------------------
// Named exports — wasm-bindgen binds these via module = "/privy-bridge.js"
// ---------------------------------------------------------------------------

/**
 * Initialise the bridge. Must be called (and awaited) before any login.
 */
export async function init(appId, clientId) {
    _appId = appId;
    privy = new Privy({ appId, clientId, storage: new LocalStorage() });
    loadSession();
    _setupEmbeddedWalletProxy();
}

/**
 * Mount the hidden Privy embedded-wallet iframe and wire up the postMessage
 * channel so that privy.embeddedWallet.getSolanaProvider() /
 * getProvider() can actually sign.  Must run in a browser context.
 */
function _setupEmbeddedWalletProxy() {
    if (_iframeReady || typeof document === "undefined") return;
    _iframeReady = true;

    const iframeUrl = privy.embeddedWallet.getURL();
    const iframe = document.createElement("iframe");
    iframe.src = iframeUrl;
    iframe.style.cssText = "display:none;position:fixed;width:0;height:0;border:0;";
    iframe.allow = "clipboard-write";
    document.body.appendChild(iframe);

    privy.setMessagePoster({
        postMessage: (data, origin) => iframe.contentWindow?.postMessage(data, origin),
        reload: () => { iframe.src = ""; iframe.src = iframeUrl; },
    });

    // Route messages from the iframe back into the SDK's handler.
    window.addEventListener("message", (event) => {
        if (event.source !== iframe.contentWindow) return;
        try {
            const data = typeof event.data === "string" ? JSON.parse(event.data) : event.data;
            if (data?.event?.startsWith?.("privy:")) {
                privy.embeddedWallet.onMessage(data);
            }
        } catch (_) { }
    });
}

// ---------------------------------------------------------------------------
// Wallet connect — called directly from JS to preserve the browser's
// user-gesture context. Rust's spawn_local breaks the gesture chain,
// causing wallet popups to be silently blocked.
// ---------------------------------------------------------------------------

/**
 * Trigger MetaMask connect popup and return the checksummed address.
 * Must be called from within a click handler (user gesture).
 * @returns {Promise<string>} checksummed Ethereum address
 */
export async function connectMetaMask() {
    const ethereum = window.ethereum;
    if (!ethereum?.isMetaMask) {
        throw new Error("MetaMask not found — install the extension");
    }
    // eth_requestAccounts triggers the popup; its return value is the
    // authorised account list (checksummed). Do NOT call eth_accounts
    // separately — that returns [] silently if not yet authorised.
    const accounts = await ethereum.request({ method: "eth_requestAccounts" });
    if (!accounts?.length) throw new Error("No account returned from MetaMask");
    return accounts[0];
}

/**
 * Trigger Phantom connect popup and return the base58 public key.
 * Must be called from within a click handler (user gesture).
 * @returns {Promise<string>} Solana public key (base58)
 */
export async function connectPhantom() {
    const solana = window.solana;
    if (!solana?.isPhantom) {
        throw new Error("Phantom not found — install the extension");
    }
    await solana.connect();
    const address = solana.publicKey?.toString();
    if (!address) throw new Error("No public key returned from Phantom");
    return address;
}

/**
 * Disconnect Phantom. MetaMask has no disconnect API — clear state in Rust.
 */
export async function disconnectPhantom() {
    const solana = window.solana;
    if (solana?.isPhantom) {
        try { await solana.disconnect(); } catch (_) { }
    }
}


// ---------------------------------------------------------------------------
// Privy embedded wallet — EVM
// Uses the EIP-1193 provider from privy.embeddedWallet.getProvider().
// The embedded wallet is available after init() when the user is logged in
// via email OTP. No browser extension needed.
// ---------------------------------------------------------------------------

async function getPrivyEvmProvider() {
    if (!privy) throw new Error("Privy not initialised");
    // privy.user is a UserApi object (not user data), so we use the REST session's
    // linked_accounts which are in the snake_case format the SDK internals expect.
    const linked = _session?.user?.linked_accounts ?? [];
    const ethWallet = linked.find(a =>
        a.type === "wallet" &&
        a.chain_type === "ethereum" &&
        (a.wallet_client_type === "privy" || a.connector_type === "embedded")
    );
    if (!ethWallet) throw new Error("No Privy embedded EVM wallet — user must be logged in via email");
    return await privy.embeddedWallet.getProvider(ethWallet);
}

/** Return the Privy embedded EVM wallet address, or null if unavailable. */
export function getPrivyEvmAddress() {
    try {
        // SDK path: populated when js-sdk-core manages auth
        const wallet = privy?.embeddedWallet;
        if (wallet?.address) return wallet.address;

        // REST-API path: address lives in the session's linked_accounts
        const linked = _session?.user?.linked_accounts ?? [];
        const evmWallet = linked.find(a =>
            a.type === "wallet" &&
            a.chain_type === "ethereum" &&
            (a.wallet_client_type === "privy" || a.connector_type === "embedded")
        );
        return evmWallet?.address ?? null;
    } catch (_) { return null; }
}

/** Sign a message with the Privy embedded EVM wallet. */
export async function signMessagePrivyEvm(message, address) {
    const provider = await getPrivyEvmProvider();
    const hex = "0x" + Array.from(new TextEncoder().encode(message))
        .map(b => b.toString(16).padStart(2, "0")).join("");
    return await provider.request({
        method: "personal_sign",
        params: [hex, address.toLowerCase()],
    });
}

/** Send native ETH via the Privy embedded EVM wallet. */
export async function sendEthPrivy(from, to, ethAmount) {
    const provider = await getPrivyEvmProvider();
    const wei = BigInt(Math.round(parseFloat(ethAmount) * 1e18));
    return await provider.request({
        method: "eth_sendTransaction",
        params: [{ from, to, value: "0x" + wei.toString(16) }],
    });
}

/** Send an ERC-20 token via the Privy embedded EVM wallet. */
export async function sendERC20Privy(from, tokenAddress, to, amount) {
    const provider = await getPrivyEvmProvider();
    const selector = "0xa9059cbb";
    const paddedTo = to.replace(/^0x/, "").toLowerCase().padStart(64, "0");
    const paddedAmt = BigInt(amount).toString(16).padStart(64, "0");
    const data = selector + paddedTo + paddedAmt;
    return await provider.request({
        method: "eth_sendTransaction",
        params: [{ from, to: tokenAddress, data }],
    });
}

/** Call a contract function via the Privy embedded EVM wallet. */
export async function callContractPrivy(from, contractAddress, data) {
    const provider = await getPrivyEvmProvider();
    return await provider.request({
        method: "eth_sendTransaction",
        params: [{ from, to: contractAddress, data }],
    });
}

// ---------------------------------------------------------------------------
// Privy embedded wallet — Solana
// ---------------------------------------------------------------------------

/**
 * Resolve the Privy embedded Solana wallet provider.
 * Uses the REST session's linked_accounts (snake_case) because privy.user in
 * sdk-core v0.60 is a UserApi object, not user data — passing it to the SDK's
 * getUserEmbeddedSolanaWallet() crashes at linked_accounts.filter().
 */
async function getPrivySolanaProvider() {
    if (!privy) throw new Error("Privy not initialised");
    const linked = _session?.user?.linked_accounts ?? [];
    const solWallet = linked.find(a =>
        a.type === "wallet" &&
        a.chain_type === "solana" &&
        (a.wallet_client_type === "privy" || a.connector_type === "embedded")
    );
    if (!solWallet) throw new Error("No Privy embedded Solana wallet — user must be logged in via email");
    const { entropyId, entropyIdVerifier } = getEntropyDetailsFromAccount(solWallet);
    return await privy.embeddedWallet.getSolanaProvider(solWallet, entropyId, entropyIdVerifier);
}

/** Return the Privy embedded Solana wallet public key string, or null. */
export function getPrivySolanaAddress() {
    try {
        // SDK path
        const w = privy?.solanaEmbeddedWallet ?? privy?.embeddedSolanaWallet;
        if (w) return w?.publicKey?.toString() ?? w?.address ?? null;

        // REST-API path
        const linked = _session?.user?.linked_accounts ?? [];
        const solWallet = linked.find(a =>
            a.type === "wallet" &&
            a.chain_type === "solana" &&
            (a.wallet_client_type === "privy" || a.connector_type === "embedded")
        );
        return solWallet?.address ?? null;
    } catch (_) { return null; }
}

/** Sign a message with the Privy embedded Solana wallet. Returns base64 signature. */
export async function signMessagePrivySolana(message) {
    const provider = await getPrivySolanaProvider();
    // The provider expects the message as base64.
    const encoded = btoa(message);
    const { signature } = await provider.request({
        method: "signMessage",
        params: { message: encoded },
    });
    return signature;
}

/** Send native SOL via the Privy embedded Solana wallet. */
export async function sendSolPrivy(toAddress, solAmount) {
    const provider = await getPrivySolanaProvider();
    const conn = new Connection(SOL_RPC, "confirmed");
    const from = new PublicKey(provider._publicKey);
    const to = new PublicKey(toAddress);
    const lamports = Math.round(parseFloat(solAmount) * LAMPORTS_PER_SOL);
    const { blockhash } = await conn.getLatestBlockhash();
    const tx = new Transaction({ recentBlockhash: blockhash, feePayer: from })
        .add(SystemProgram.transfer({ fromPubkey: from, toPubkey: to, lamports }));
    const { signature } = await provider.request({
        method: "signAndSendTransaction",
        params: { transaction: tx, connection: conn },
    });
    return signature;
}

/** Send an SPL token via the Privy embedded Solana wallet. */
export async function sendSPLTokenPrivy(toAddress, mintAddress, amount) {
    const provider = await getPrivySolanaProvider();
    const conn = new Connection(SOL_RPC, "confirmed");
    const from = new PublicKey(provider._publicKey);
    const mint = new PublicKey(mintAddress);
    const toPk = new PublicKey(toAddress);
    const fromATA = await getAssociatedTokenAddress(mint, from);
    const toATA = await getAssociatedTokenAddress(mint, toPk);
    const { blockhash } = await conn.getLatestBlockhash();
    const tx = new Transaction({ recentBlockhash: blockhash, feePayer: from })
        .add(createTransferInstruction(fromATA, toATA, from, BigInt(amount)));
    const { signature } = await provider.request({
        method: "signAndSendTransaction",
        params: { transaction: tx, connection: conn },
    });
    return signature;
}

/** Invoke a Solana program via the Privy embedded Solana wallet. */
export async function invokeProgramPrivy(programId, dataHex, accountsMeta) {
    const { TransactionInstruction } = await import("@solana/web3.js");
    const provider = await getPrivySolanaProvider();
    const conn = new Connection(SOL_RPC, "confirmed");
    const feePayer = new PublicKey(provider._publicKey);
    const data = Buffer.from(dataHex, "hex");
    const keys = accountsMeta.map((pk, i) => ({
        pubkey: new PublicKey(pk), isSigner: i === 0, isWritable: i < 2,
    }));
    const ix = new TransactionInstruction({ programId: new PublicKey(programId), keys, data });
    const { blockhash } = await conn.getLatestBlockhash();
    const tx = new Transaction({ recentBlockhash: blockhash, feePayer }).add(ix);
    const { signature } = await provider.request({
        method: "signAndSendTransaction",
        params: { transaction: tx, connection: conn },
    });
    return signature;
}

// ---------------------------------------------------------------------------
// MetaMask — wallet actions
// All functions call ethereum.request() synchronously (before first await) so
// the Promise is created within the user-gesture context → popup is allowed.
// ---------------------------------------------------------------------------

/** Sign an arbitrary message with MetaMask (personal_sign). */
export async function signMessageMetaMask(message, address) {
    const eth = window.ethereum;
    if (!eth?.isMetaMask) throw new Error("MetaMask not found");
    const hex = "0x" + Array.from(new TextEncoder().encode(message))
        .map(b => b.toString(16).padStart(2, "0")).join("");
    return await eth.request({
        method: "personal_sign",
        params: [hex, address.toLowerCase()],
    });
}

/**
 * Send native ETH.
 * @param {string} from      — sender address
 * @param {string} to        — recipient address
 * @param {string} ethAmount — human-readable amount e.g. "0.01"
 * @returns {Promise<string>} tx hash
 */
export async function sendEthMetaMask(from, to, ethAmount) {
    const eth = window.ethereum;
    if (!eth?.isMetaMask) throw new Error("MetaMask not found");
    const wei = BigInt(Math.round(parseFloat(ethAmount) * 1e18));
    const valueHex = "0x" + wei.toString(16);
    return await eth.request({
        method: "eth_sendTransaction",
        params: [{ from, to, value: valueHex }],
    });
}

/**
 * Send an ERC-20 token.
 * ABI-encodes transfer(address,uint256) without an external library.
 * @param {string} from         — sender address
 * @param {string} tokenAddress — ERC-20 contract address
 * @param {string} to           — recipient address
 * @param {string} amount       — raw token units (no decimals applied)
 * @returns {Promise<string>} tx hash
 */
export async function sendERC20MetaMask(from, tokenAddress, to, amount) {
    const eth = window.ethereum;
    if (!eth?.isMetaMask) throw new Error("MetaMask not found");
    const selector = "0xa9059cbb"; // transfer(address,uint256)
    const paddedTo = to.replace(/^0x/, "").toLowerCase().padStart(64, "0");
    const paddedAmt = BigInt(amount).toString(16).padStart(64, "0");
    const data = selector + paddedTo + paddedAmt;
    return await eth.request({
        method: "eth_sendTransaction",
        params: [{ from, to: tokenAddress, data }],
    });
}

/**
 * Call an arbitrary contract function.
 * @param {string} from             — sender address
 * @param {string} contractAddress  — contract address
 * @param {string} data             — ABI-encoded calldata (0x-prefixed hex)
 * @returns {Promise<string>} tx hash
 */
export async function callContractMetaMask(from, contractAddress, data) {
    const eth = window.ethereum;
    if (!eth?.isMetaMask) throw new Error("MetaMask not found");
    return await eth.request({
        method: "eth_sendTransaction",
        params: [{ from, to: contractAddress, data }],
    });
}

// ---------------------------------------------------------------------------
// Phantom — wallet actions
// ---------------------------------------------------------------------------

const SOL_RPC = "https://api.mainnet-beta.solana.com"; // swap to devnet for testing

/** Sign an arbitrary message with Phantom. Returns base64-encoded signature. */
export async function signMessagePhantom(message) {
    const sol = window.solana;
    if (!sol?.isPhantom) throw new Error("Phantom not found");
    const encoded = new TextEncoder().encode(message);
    const { signature } = await sol.signMessage(encoded, "utf8");
    let binary = "";
    for (let i = 0; i < signature.byteLength; i++) binary += String.fromCharCode(signature[i]);
    return btoa(binary);
}

/**
 * Send native SOL.
 * @param {string} toAddress — recipient base58 public key
 * @param {string} solAmount — human-readable amount e.g. "0.01"
 * @returns {Promise<string>} transaction signature
 */
export async function sendSolPhantom(toAddress, solAmount) {
    const sol = window.solana;
    if (!sol?.isPhantom) throw new Error("Phantom not found");
    const conn = new Connection(SOL_RPC, "confirmed");
    const from = new PublicKey(sol.publicKey.toString());
    const to = new PublicKey(toAddress);
    const lamports = Math.round(parseFloat(solAmount) * LAMPORTS_PER_SOL);
    const { blockhash } = await conn.getLatestBlockhash();
    const tx = new Transaction({ recentBlockhash: blockhash, feePayer: from })
        .add(SystemProgram.transfer({ fromPubkey: from, toPubkey: to, lamports }));
    const { signature } = await sol.signAndSendTransaction(tx);
    return signature;
}

/**
 * Send an SPL token.
 * @param {string} toAddress   — recipient base58 public key
 * @param {string} mintAddress — token mint address
 * @param {string} amount      — raw token units (no decimals applied)
 * @returns {Promise<string>} transaction signature
 */
export async function sendSPLTokenPhantom(toAddress, mintAddress, amount) {
    const sol = window.solana;
    if (!sol?.isPhantom) throw new Error("Phantom not found");
    const conn = new Connection(SOL_RPC, "confirmed");
    const from = new PublicKey(sol.publicKey.toString());
    const mint = new PublicKey(mintAddress);
    const toPk = new PublicKey(toAddress);
    const fromATA = await getAssociatedTokenAddress(mint, from);
    const toATA = await getAssociatedTokenAddress(mint, toPk);
    const { blockhash } = await conn.getLatestBlockhash();
    const tx = new Transaction({ recentBlockhash: blockhash, feePayer: from })
        .add(createTransferInstruction(fromATA, toATA, from, BigInt(amount)));
    const { signature } = await sol.signAndSendTransaction(tx);
    return signature;
}

/**
 * Invoke a Solana program (generic instruction).
 * @param {string} programId — program address
 * @param {string} dataHex   — hex-encoded instruction data (no 0x prefix)
 * @param {string[]} accountsMeta — array of base58 account pubkeys (writable signers first)
 * @returns {Promise<string>} transaction signature
 */
export async function invokeProgramPhantom(programId, dataHex, accountsMeta) {
    const sol = window.solana;
    if (!sol?.isPhantom) throw new Error("Phantom not found");
    const { TransactionInstruction } = await import("@solana/web3.js");
    const conn = new Connection(SOL_RPC, "confirmed");
    const feePayer = new PublicKey(sol.publicKey.toString());
    const data = Buffer.from(dataHex, "hex");
    const keys = accountsMeta.map((pk, i) => ({
        pubkey: new PublicKey(pk),
        isSigner: i === 0,
        isWritable: i < 2,
    }));
    const ix = new TransactionInstruction({ programId: new PublicKey(programId), keys, data });
    const { blockhash } = await conn.getLatestBlockhash();
    const tx = new Transaction({ recentBlockhash: blockhash, feePayer }).add(ix);
    const { signature } = await sol.signAndSendTransaction(tx);
    return signature;
}

// ---------------------------------------------------------------------------
// Counter Contract — EVM (Hardhat/Ignition)
// inc(), incBy(uint256), x() → current count
// ---------------------------------------------------------------------------

const INC_SELECTOR    = "0x371303c0"; // keccak256("inc()")[0:4]
const INC_BY_SELECTOR = "0x70119d06"; // keccak256("incBy(uint256)")[0:4]
const X_SELECTOR      = "0x0c55699c"; // keccak256("x()")[0:4]

/** Deployed Counter addresses keyed by chain ID (decimal string). */
const COUNTER_ADDRESSES = {
    "11155111": "0xa8a8230679de251F92fE47EF2B8cc60558430765", // Sepolia
    "80002":    "0x2C6f788b05A4A7799c7892F62b951290ff882B2b", // Amoy
};

/**
 * Return the deployed Counter address for the currently connected chain,
 * or null if the chain is not supported.
 */
export async function getCounterAddress() {
    const provider = window.ethereum ?? (privy ? getPrivyEvmProvider() : null);
    if (!provider) return null;
    try {
        const chainIdHex = await provider.request({ method: "eth_chainId" });
        const chainId = String(parseInt(chainIdHex, 16));
        return COUNTER_ADDRESSES[chainId] ?? null;
    } catch (_) { return null; }
}

/** Read the current counter value via eth_call (no gas). */
export async function readCounterEVM(contractAddress) {
    let provider;
    if (window.ethereum) {
        provider = window.ethereum;
    } else {
        provider = await getPrivyEvmProvider();
    }
    const result = await provider.request({
        method: "eth_call",
        params: [{ to: contractAddress, data: X_SELECTOR }, "latest"],
    });
    return BigInt(result).toString();
}

/** Call Counter.inc() via MetaMask. */
export async function counterIncMetaMask(from, contractAddress) {
    const eth = window.ethereum;
    if (!eth?.isMetaMask) throw new Error("MetaMask not found");
    return await eth.request({
        method: "eth_sendTransaction",
        params: [{ from, to: contractAddress, data: INC_SELECTOR }],
    });
}

/** Call Counter.incBy(uint256) via MetaMask. */
export async function counterIncByMetaMask(from, contractAddress, by) {
    const eth = window.ethereum;
    if (!eth?.isMetaMask) throw new Error("MetaMask not found");
    const data = INC_BY_SELECTOR + BigInt(by).toString(16).padStart(64, "0");
    return await eth.request({
        method: "eth_sendTransaction",
        params: [{ from, to: contractAddress, data }],
    });
}

/** Call Counter.inc() via the Privy embedded EVM wallet. */
export async function counterIncPrivy(from, contractAddress) {
    const provider = await getPrivyEvmProvider();
    return await provider.request({
        method: "eth_sendTransaction",
        params: [{ from, to: contractAddress, data: INC_SELECTOR }],
    });
}

/** Call Counter.incBy(uint256) via the Privy embedded EVM wallet. */
export async function counterIncByPrivy(from, contractAddress, by) {
    const provider = await getPrivyEvmProvider();
    const data = INC_BY_SELECTOR + BigInt(by).toString(16).padStart(64, "0");
    return await provider.request({
        method: "eth_sendTransaction",
        params: [{ from, to: contractAddress, data }],
    });
}

// ---------------------------------------------------------------------------
// Counter Program — Solana (Anchor)
// Program: BHqZnZvzQ9ogmBJKjSzKEukqwxXCrgWgqsaDNbU6QkGw
// PDA seeds: [b"counter"]
// ---------------------------------------------------------------------------

const COUNTER_SOL_RPC = "https://api.devnet.solana.com";

/** Compute an 8-byte Anchor instruction discriminator: sha256("global:<name>")[0:8] */
async function anchorDiscriminator(name) {
    const data = new TextEncoder().encode("global:" + name);
    const hashBuf = await crypto.subtle.digest("SHA-256", data);
    return Buffer.from(new Uint8Array(hashBuf).slice(0, 8));
}

/** Derive the counter PDA (seeds: [b"counter"]) for a given program. */
function deriveCounterPda(programId) {
    const [pda] = PublicKey.findProgramAddressSync(
        [Buffer.from("counter")],
        new PublicKey(programId)
    );
    return pda;
}

/** Read the current counter value from the Solana program account. */
export async function readCounterSolana(programId) {
    const conn = new Connection(COUNTER_SOL_RPC, "confirmed");
    const pda = deriveCounterPda(programId);
    const info = await conn.getAccountInfo(pda);
    if (!info) return "not initialized";
    // Anchor account layout: 8-byte discriminator + u64 (little-endian)
    const count = info.data.readBigUInt64LE(8);
    return count.toString();
}

/** Initialize the Solana counter via Phantom. */
export async function counterInitializePhantom(programId) {
    const { TransactionInstruction } = await import("@solana/web3.js");
    const sol = window.solana;
    if (!sol?.isPhantom) throw new Error("Phantom not found");
    const conn = new Connection(COUNTER_SOL_RPC, "confirmed");
    const payer = new PublicKey(sol.publicKey.toString());
    const progPk = new PublicKey(programId);
    const pda = deriveCounterPda(programId);
    const disc = await anchorDiscriminator("initialize");
    const ix = new TransactionInstruction({
        programId: progPk,
        keys: [
            { pubkey: payer, isSigner: true,  isWritable: true  },
            { pubkey: pda,   isSigner: false, isWritable: true  },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: disc,
    });
    const { blockhash } = await conn.getLatestBlockhash();
    const tx = new Transaction({ recentBlockhash: blockhash, feePayer: payer }).add(ix);
    const { signature } = await sol.signAndSendTransaction(tx);
    return signature;
}

/** Increment the Solana counter via Phantom. */
export async function counterIncrementPhantom(programId) {
    const { TransactionInstruction } = await import("@solana/web3.js");
    const sol = window.solana;
    if (!sol?.isPhantom) throw new Error("Phantom not found");
    const conn = new Connection(COUNTER_SOL_RPC, "confirmed");
    const payer = new PublicKey(sol.publicKey.toString());
    const progPk = new PublicKey(programId);
    const pda = deriveCounterPda(programId);
    const disc = await anchorDiscriminator("increment");
    const ix = new TransactionInstruction({
        programId: progPk,
        keys: [{ pubkey: pda, isSigner: false, isWritable: true }],
        data: disc,
    });
    const { blockhash } = await conn.getLatestBlockhash();
    const tx = new Transaction({ recentBlockhash: blockhash, feePayer: payer }).add(ix);
    const { signature } = await sol.signAndSendTransaction(tx);
    return signature;
}

/** Initialize the Solana counter via the Privy embedded Solana wallet. */
export async function counterInitializePrivy(programId) {
    const { TransactionInstruction } = await import("@solana/web3.js");
    const provider = await getPrivySolanaProvider();
    const conn = new Connection(COUNTER_SOL_RPC, "confirmed");
    const payer = new PublicKey(provider._publicKey);
    const progPk = new PublicKey(programId);
    const pda = deriveCounterPda(programId);
    const disc = await anchorDiscriminator("initialize");
    const ix = new TransactionInstruction({
        programId: progPk,
        keys: [
            { pubkey: payer, isSigner: true,  isWritable: true  },
            { pubkey: pda,   isSigner: false, isWritable: true  },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: disc,
    });
    const { blockhash } = await conn.getLatestBlockhash();
    const tx = new Transaction({ recentBlockhash: blockhash, feePayer: payer }).add(ix);
    const { signature } = await provider.request({
        method: "signAndSendTransaction",
        params: { transaction: tx, connection: conn },
    });
    return signature;
}

/** Increment the Solana counter via the Privy embedded Solana wallet. */
export async function counterIncrementPrivy(programId) {
    const { TransactionInstruction } = await import("@solana/web3.js");
    const provider = await getPrivySolanaProvider();
    const conn = new Connection(COUNTER_SOL_RPC, "confirmed");
    const payer = new PublicKey(provider._publicKey);
    const progPk = new PublicKey(programId);
    const pda = deriveCounterPda(programId);
    const disc = await anchorDiscriminator("increment");
    const ix = new TransactionInstruction({
        programId: progPk,
        keys: [{ pubkey: pda, isSigner: false, isWritable: true }],
        data: disc,
    });
    const { blockhash } = await conn.getLatestBlockhash();
    const tx = new Transaction({ recentBlockhash: blockhash, feePayer: payer }).add(ix);
    const { signature } = await provider.request({
        method: "signAndSendTransaction",
        params: { transaction: tx, connection: conn },
    });
    return signature;
}

// ---------------------------------------------------------------------------
// Email OTP  (Privy passwordless flow)
// Uses the SDK's privy.auth.email methods so that ALL token types
// (token, privy_access_token, refresh_token) are stored via
// session.updateWithTokensResponse().  This is required for the embedded
// wallet iframe to accept the access token when signing.
// ---------------------------------------------------------------------------

/**
 * Step 1 — send a 6-digit OTP to the user's email.
 */
export async function sendEmailCode(email) {
    assertReady();
    // SDK manages the flow_context internally.
    await privy.auth.email.sendCode(email);
}

/**
 * Step 2 — verify the OTP and exchange it for a session.
 * privy.auth.email.loginWithCode() calls session.updateWithTokensResponse()
 * which stores privy:token, privy:pat, and privy:refresh_token so that
 * getAccessTokenInternal() returns a token the embedded wallet iframe accepts.
 * @returns serialised user object
 */
export async function verifyEmailCode(email, code) {
    assertReady();
    const result = await privy.auth.email.loginWithCode(email, code.trim(), "login-or-sign-up");
    // Keep _session.user so getPrivyEvmAddress / getPrivySolanaAddress work.
    // The SDK result.user has linked_accounts in snake_case (raw API format).
    saveSession({ user: result.user });
    return serializeUser(result.user);
}


export async function logout() {
    assertReady();
    clearSession();
    try { await privy.auth.logout(); } catch (_) { }
}

export function getUser() {
    // _session.user is set for both REST-auth (MetaMask/Phantom) and SDK-auth (email) users.
    if (_session?.user) return serializeUser(_session.user);
    return null;
}

export function isAuthenticated() {
    // _session.user is always populated after any login path.
    return !!_session?.user;
}

export function getAccessToken() {
    // For REST-auth (MetaMask/Phantom) sessions, token lives in _session.
    // For SDK-auth (email) sessions, the SDK manages the token internally.
    return _session?.token ?? null;
}