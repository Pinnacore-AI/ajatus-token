// Ajatuskumppani Token (AJT) - Solana SPL Token Program
// Built with Anchor framework

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("AJT11111111111111111111111111111111111111111");

#[program]
pub mod ajt_token {
    use super::*;

    /// Initialize the AJT token
    pub fn initialize(
        ctx: Context<Initialize>,
        total_supply: u64,
    ) -> Result<()> {
        let token_info = &mut ctx.accounts.token_info;
        token_info.authority = ctx.accounts.authority.key();
        token_info.total_supply = total_supply;
        token_info.circulating_supply = 0;
        token_info.bump = *ctx.bumps.get("token_info").unwrap();
        
        msg!("AJT Token initialized with supply: {}", total_supply);
        Ok(())
    }

    /// Mint new tokens (only authority)
    pub fn mint_tokens(
        ctx: Context<MintTokens>,
        amount: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.token_info.circulating_supply + amount <= ctx.accounts.token_info.total_supply,
            ErrorCode::ExceedsMaxSupply
        );

        // Mint tokens
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, amount)?;

        // Update circulating supply
        ctx.accounts.token_info.circulating_supply += amount;

        msg!("Minted {} AJT tokens", amount);
        Ok(())
    }

    /// Stake tokens
    pub fn stake(
        ctx: Context<Stake>,
        amount: u64,
    ) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;
        let clock = Clock::get()?;

        // Transfer tokens to stake vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.stake_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Update stake account
        stake_account.user = ctx.accounts.user.key();
        stake_account.amount += amount;
        stake_account.last_stake_time = clock.unix_timestamp;

        msg!("Staked {} AJT tokens", amount);
        Ok(())
    }

    /// Unstake tokens
    pub fn unstake(
        ctx: Context<Unstake>,
        amount: u64,
    ) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;
        
        require!(
            stake_account.amount >= amount,
            ErrorCode::InsufficientStake
        );

        // Calculate rewards before unstaking
        let rewards = calculate_rewards(stake_account)?;

        // Transfer tokens back to user
        let seeds = &[
            b"stake_vault",
            &[ctx.accounts.token_info.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.stake_vault.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.token_info.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount + rewards)?;

        // Update stake account
        stake_account.amount -= amount;
        stake_account.total_rewards += rewards;

        msg!("Unstaked {} AJT tokens + {} rewards", amount, rewards);
        Ok(())
    }

    /// Claim staking rewards
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;
        let rewards = calculate_rewards(stake_account)?;

        require!(rewards > 0, ErrorCode::NoRewards);

        // Transfer rewards to user
        let seeds = &[
            b"stake_vault",
            &[ctx.accounts.token_info.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.stake_vault.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.token_info.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, rewards)?;

        // Update stake account
        stake_account.total_rewards += rewards;
        stake_account.last_claim_time = Clock::get()?.unix_timestamp;

        msg!("Claimed {} AJT rewards", rewards);
        Ok(())
    }
}

// Helper function to calculate staking rewards
fn calculate_rewards(stake_account: &StakeAccount) -> Result<u64> {
    let clock = Clock::get()?;
    let time_staked = clock.unix_timestamp - stake_account.last_stake_time;
    
    // 10% APY = 0.1 / (365 * 24 * 60 * 60) per second
    let apy_per_second = 0.1 / 31_536_000.0;
    let rewards = (stake_account.amount as f64 * apy_per_second * time_staked as f64) as u64;
    
    Ok(rewards)
}

// Account structures

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + TokenInfo::LEN,
        seeds = [b"token_info"],
        bump
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(
        mut,
        seeds = [b"token_info"],
        bump = token_info.bump
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + StakeAccount::LEN,
        seeds = [b"stake", user.key().as_ref()],
        bump
    )]
    pub stake_account: Account<'info, StakeAccount>,
    
    #[account(
        mut,
        seeds = [b"token_info"],
        bump = token_info.bump
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub stake_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(
        mut,
        seeds = [b"stake", user.key().as_ref()],
        bump
    )]
    pub stake_account: Account<'info, StakeAccount>,
    
    #[account(
        mut,
        seeds = [b"token_info"],
        bump = token_info.bump
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    #[account(mut)]
    pub stake_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(
        mut,
        seeds = [b"stake", user.key().as_ref()],
        bump
    )]
    pub stake_account: Account<'info, StakeAccount>,
    
    #[account(
        mut,
        seeds = [b"token_info"],
        bump = token_info.bump
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    #[account(mut)]
    pub stake_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

// Data structures

#[account]
pub struct TokenInfo {
    pub authority: Pubkey,
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub bump: u8,
}

impl TokenInfo {
    pub const LEN: usize = 32 + 8 + 8 + 1;
}

#[account]
pub struct StakeAccount {
    pub user: Pubkey,
    pub amount: u64,
    pub last_stake_time: i64,
    pub last_claim_time: i64,
    pub total_rewards: u64,
}

impl StakeAccount {
    pub const LEN: usize = 32 + 8 + 8 + 8 + 8;
}

// Error codes

#[error_code]
pub enum ErrorCode {
    #[msg("Exceeds maximum token supply")]
    ExceedsMaxSupply,
    
    #[msg("Insufficient stake amount")]
    InsufficientStake,
    
    #[msg("No rewards available to claim")]
    NoRewards,
}

