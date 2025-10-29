# AJT Token Deployment Guide

Ohjeet Ajatuskumppani (AJT) SPL Token -sopimuksen deployaamiseen Solanaan.

## Esivalmistelut

### 1. Asenna Solana CLI

```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

### 2. Asenna Anchor

```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest
```

### 3. Luo Solana Wallet

```bash
solana-keygen new --outfile ~/.config/solana/id.json
```

Tallenna seed phrase turvalliseen paikkaan!

## Devnet Deployment

### 1. Konfiguroi Devnet

```bash
solana config set --url devnet
```

### 2. Hanki Devnet SOL

```bash
solana airdrop 2
```

### 3. Build Program

```bash
anchor build
```

### 4. Deploy Devnet

```bash
anchor deploy
```

### 5. Testaa

```bash
anchor test
```

## Mainnet Deployment

### 1. Konfiguroi Mainnet

```bash
solana config set --url mainnet-beta
```

### 2. Hanki SOL

Siirrä riittävästi SOL wallettiisi deployment-kuluja varten (~5-10 SOL).

### 3. Update Program ID

Päivitä `Anchor.toml` ja `lib.rs` oikealla program ID:llä:

```bash
anchor keys list
```

### 4. Build Production

```bash
anchor build --verifiable
```

### 5. Deploy Mainnet

```bash
anchor deploy --provider.cluster mainnet
```

### 6. Verify Deployment

```bash
solana program show <PROGRAM_ID>
```

## Token Initialization

### 1. Initialize Token

```bash
anchor run initialize-token
```

### 2. Mint Initial Supply

```bash
anchor run mint-initial-supply
```

### 3. Setup Distribution

Jaa tokenit seuraavasti:

- **40% Community Rewards**: 400,000,000 AJT
- **25% Development Fund**: 250,000,000 AJT
- **15% Ecosystem Growth**: 150,000,000 AJT
- **10% Team & Advisors**: 100,000,000 AJT (1 vuoden vesting)
- **10% Public Sale**: 100,000,000 AJT

## Post-Deployment

### 1. Verify on Solscan

Tarkista deployment: https://solscan.io/token/<TOKEN_ADDRESS>

### 2. Add to Jupiter

Listaa token Jupiteriin likviditeettiä varten.

### 3. Create Liquidity Pool

Luo likviditeettipooli Raydiumissa tai Orcassa:

```bash
# Esimerkki Raydium
AJT/USDC pool
Initial liquidity: 100,000 AJT + 10,000 USDC
```

### 4. Update Documentation

Päivitä dokumentaatio oikeilla osoitteilla:
- Token address
- Program ID
- Pool addresses

## Security Checklist

- [ ] Audit smart contract
- [ ] Test on devnet thoroughly
- [ ] Verify program is immutable after deployment
- [ ] Secure authority keys
- [ ] Setup multisig for critical operations
- [ ] Monitor token supply
- [ ] Setup alerts for unusual activity

## Useful Commands

```bash
# Check balance
solana balance

# Check token supply
spl-token supply <TOKEN_ADDRESS>

# Check token account
spl-token accounts

# Transfer tokens
spl-token transfer <TOKEN_ADDRESS> <AMOUNT> <RECIPIENT>

# Burn tokens
spl-token burn <TOKEN_ADDRESS> <AMOUNT>
```

## Troubleshooting

### Insufficient SOL

```bash
# Check balance
solana balance

# Get more SOL (devnet)
solana airdrop 2
```

### Program Deploy Failed

```bash
# Increase compute units
anchor deploy --provider.cluster devnet -- --max-sign-attempts 100
```

### Token Already Exists

```bash
# Use existing token address
# Update Anchor.toml with correct address
```

## Resources

- [Solana Docs](https://docs.solana.com/)
- [Anchor Docs](https://www.anchor-lang.com/)
- [SPL Token](https://spl.solana.com/token)
- [Solscan](https://solscan.io/)

## Support

- Telegram: https://t.me/ajatuskumppani
- Discord: https://discord.gg/z53hngJHd
- Email: gronmark@pinnacore.ai

