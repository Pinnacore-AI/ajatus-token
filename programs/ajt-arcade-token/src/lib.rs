use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Burn};

declare_id!("AJTxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

#[program]
pub mod ajt_arcade_token {
    use super::*;

    /// Initialize the AJT Arcade Token program
    pub fn initialize(
        ctx: Context<Initialize>,
        target_price_usd: u64, // Price in micro-USD (e.g., 1000 = $0.001)
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.mint = ctx.accounts.mint.key();
        config.target_price_usd = target_price_usd;
        config.total_minted = 0;
        config.total_burned = 0;
        config.is_paused = false;
        
        msg!("AJT Arcade Token initialized with target price: ${}", target_price_usd as f64 / 1_000_000.0);
        Ok(())
    }

    /// Mint new AJT tokens (on-demand minting for purchases)
    pub fn mint_tokens(
        ctx: Context<MintTokens>,
        amount: u64,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        
        require!(!config.is_paused, ErrorCode::ProgramPaused);
        require!(amount > 0, ErrorCode::InvalidAmount);

        // Mint tokens to user
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::mint_to(cpi_ctx, amount)?;

        // Update stats
        config.total_minted = config.total_minted.checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        msg!("Minted {} AJT tokens to user", amount);
        Ok(())
    }

    /// Burn AJT tokens (when user consumes services)
    pub fn burn_tokens(
        ctx: Context<BurnTokens>,
        amount: u64,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        
        require!(!config.is_paused, ErrorCode::ProgramPaused);
        require!(amount > 0, ErrorCode::InvalidAmount);

        // Burn tokens from user
        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::burn(cpi_ctx, amount)?;

        // Update stats
        config.total_burned = config.total_burned.checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        msg!("Burned {} AJT tokens from user", amount);
        Ok(())
    }

    /// Purchase service with AJT tokens (burn + emit event)
    pub fn purchase_service(
        ctx: Context<PurchaseService>,
        service_type: ServiceType,
        amount: u64,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        
        require!(!config.is_paused, ErrorCode::ProgramPaused);
        require!(amount > 0, ErrorCode::InvalidAmount);

        // Burn tokens
        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::burn(cpi_ctx, amount)?;

        // Update stats
        config.total_burned = config.total_burned.checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        // Emit event
        emit!(ServicePurchased {
            user: ctx.accounts.user.key(),
            service_type,
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        msg!("User purchased {:?} service for {} AJT", service_type, amount);
        Ok(())
    }

    /// Update target price (admin only)
    pub fn update_price(
        ctx: Context<UpdateConfig>,
        new_price_usd: u64,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.target_price_usd = new_price_usd;
        
        msg!("Updated target price to: ${}", new_price_usd as f64 / 1_000_000.0);
        Ok(())
    }

    /// Pause/unpause the program (emergency)
    pub fn set_pause(
        ctx: Context<UpdateConfig>,
        paused: bool,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.is_paused = paused;
        
        msg!("Program paused: {}", paused);
        Ok(())
    }

    /// Get current stats
    pub fn get_stats(ctx: Context<GetStats>) -> Result<TokenStats> {
        let config = &ctx.accounts.config;
        
        Ok(TokenStats {
            total_minted: config.total_minted,
            total_burned: config.total_burned,
            circulating_supply: config.total_minted.saturating_sub(config.total_burned),
            target_price_usd: config.target_price_usd,
            is_paused: config.is_paused,
        })
    }
}

// Accounts

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Config::LEN,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        mint::decimals = 9,
        mint::authority = authority,
    )]
    pub mint: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump,
        has_one = authority,
        has_one = mint,
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump,
        has_one = mint,
    )]
    pub config: Account<'info, Config>,
    
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PurchaseService<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump,
        has_one = mint,
    )]
    pub config: Account<'info, Config>,
    
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump,
        has_one = authority,
    )]
    pub config: Account<'info, Config>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetStats<'info> {
    #[account(
        seeds = [b"config"],
        bump,
    )]
    pub config: Account<'info, Config>,
}

// State

#[account]
pub struct Config {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub target_price_usd: u64, // Micro-USD (1_000_000 = $1)
    pub total_minted: u64,
    pub total_burned: u64,
    pub is_paused: bool,
}

impl Config {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 1;
}

// Events

#[event]
pub struct ServicePurchased {
    pub user: Pubkey,
    pub service_type: ServiceType,
    pub amount: u64,
    pub timestamp: i64,
}

// Enums

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ServiceType {
    Chat,           // Chat message (1K tokens)
    CodeExecution,  // Code run
    RagDocument,    // Document upload
    ImageGen,       // Image generation
    AgentEvolution, // Agent evolution
    VoiceSynthesis, // Voice synthesis
}

// Return types

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TokenStats {
    pub total_minted: u64,
    pub total_burned: u64,
    pub circulating_supply: u64,
    pub target_price_usd: u64,
    pub is_paused: bool,
}

// Errors

#[error_code]
pub enum ErrorCode {
    #[msg("Program is paused")]
    ProgramPaused,
    
    #[msg("Invalid amount")]
    InvalidAmount,
    
    #[msg("Arithmetic overflow")]
    Overflow,
}

