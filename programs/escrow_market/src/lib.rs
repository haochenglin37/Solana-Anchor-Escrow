use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer};

declare_id!("Escrow1111111111111111111111111111111111111");

pub const FEE_BPS: u16 = 100; // 1%
pub const ADMIN_PUBKEY: Pubkey = pubkey!("FEEAdm11111111111111111111111111111111111");

#[program]
pub mod escrow_market {
    use super::*;

    pub fn create_listing(
        ctx: Context<CreateListing>,
        price: u64,
        quantity: u64,
        expiry: i64,
        nonce: u64,
    ) -> Result<()> {
        require!(quantity > 0, EscrowError::InvalidQuantity);
        require!(price > 0, EscrowError::InvalidPrice);

        token::transfer(
            ctx.accounts.transfer_to_escrow_context(),
            quantity,
        )?;

        let listing = &mut ctx.accounts.listing;
        listing.seller = ctx.accounts.seller.key();
        listing.mint = ctx.accounts.mint.key();
        listing.price = price;
        listing.quantity = quantity;
        listing.expiry = expiry;
        listing.status = ListingStatus::Open as u8;
        listing.bump = *ctx.bumps.get("listing").unwrap();
        listing.nonce = nonce;

        emit!(ListingCreated {
            seller: listing.seller,
            listing: listing.key(),
            mint: listing.mint,
            price,
            quantity,
        });

        Ok(())
    }

    pub fn buy(ctx: Context<Buy>) -> Result<()> {
        let listing = &mut ctx.accounts.listing;
        require!(
            listing.status == ListingStatus::Open as u8,
            EscrowError::InvalidState
        );
        listing.status = ListingStatus::Settled as u8;

        let fee = listing
            .price
            .checked_mul(FEE_BPS as u64)
            .ok_or(EscrowError::Overflow)?
            .checked_div(10_000)
            .ok_or(EscrowError::Overflow)?;
        let seller_amount = listing
            .price
            .checked_sub(fee)
            .ok_or(EscrowError::Overflow)?;

        token::transfer(ctx.accounts.pay_seller_context(), seller_amount)?;
        token::transfer(ctx.accounts.pay_fee_context(), fee)?;
        token::transfer(
            ctx.accounts
                .escrow_to_buyer_context()
                .with_signer(&[ctx.accounts.listing_seeds()]),
            listing.quantity,
        )?;
        token::close_account(
            ctx.accounts
                .close_escrow_context()
                .with_signer(&[ctx.accounts.listing_seeds()]),
        )?;

        emit!(Purchased {
            listing: listing.key(),
            buyer: ctx.accounts.buyer.key(),
            price: listing.price,
        });

        Ok(())
    }

    pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
        let listing = &mut ctx.accounts.listing;
        require!(
            listing.status == ListingStatus::Open as u8,
            EscrowError::InvalidState
        );
        let now = Clock::get()?.unix_timestamp;
        require!(
            ctx.accounts.authority.key() == listing.seller || now > listing.expiry,
            EscrowError::Unauthorized
        );

        listing.status = ListingStatus::Canceled as u8;

        token::transfer(
            ctx.accounts
                .escrow_to_seller_context()
                .with_signer(&[ctx.accounts.listing_seeds()]),
            listing.quantity,
        )?;
        token::close_account(
            ctx.accounts
                .close_escrow_context()
                .with_signer(&[ctx.accounts.listing_seeds()]),
        )?;

        emit!(Canceled {
            listing: listing.key(),
            seller: listing.seller,
        });

        Ok(())
    }

    pub fn withdraw_fee(ctx: Context<WithdrawFee>, amount: u64) -> Result<()> {
        require_keys_eq!(ctx.accounts.admin.key(), ADMIN_PUBKEY, EscrowError::Unauthorized);

        token::transfer(
            ctx.accounts
                .vault_to_admin_context()
                .with_signer(&[&[b"fee_vault", &[ctx.bumps.fee_vault_authority]]]),
            amount,
        )?;

        emit!(FeeWithdrawn {
            admin: ctx.accounts.admin.key(),
            amount,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(price: u64, quantity: u64, expiry: i64, nonce: u64)]
pub struct CreateListing<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(
        init,
        payer = seller,
        space = Listing::LEN,
        seeds = [b"listing", seller.key().as_ref(), mint.key().as_ref(), &nonce.to_le_bytes()],
        bump
    )]
    pub listing: Account<'info, Listing>,
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = seller_token_account.owner == seller.key(),
        constraint = seller_token_account.mint == mint.key(),
    )]
    pub seller_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = listing,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateListing<'info> {
    fn transfer_to_escrow_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.seller_token_account.to_account_info(),
            to: self.escrow_token_account.to_account_info(),
            authority: self.seller.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub listing: Account<'info, Listing>,
    #[account(
        mut,
        constraint = buyer_payment_account.owner == buyer.key(),
        constraint = buyer_payment_account.mint == listing.mint
    )]
    pub buyer_payment_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = buyer_receive_account.owner == buyer.key(),
        constraint = buyer_receive_account.mint == listing.mint
    )]
    pub buyer_receive_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = seller_receiving_account.owner == listing.seller,
        constraint = seller_receiving_account.mint == listing.mint
    )]
    pub seller_receiving_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = listing.mint,
        associated_token::authority = listing,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"fee_vault"],
        bump,
    )]
    pub fee_vault: Account<'info, TokenAccount>,
    /// CHECK: PDA authority for fee vault
    #[account(seeds = [b"fee_vault"], bump)]
    pub fee_vault_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Buy<'info> {
    fn pay_seller_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.buyer_payment_account.to_account_info(),
            to: self.seller_receiving_account.to_account_info(),
            authority: self.buyer.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
    fn pay_fee_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.buyer_payment_account.to_account_info(),
            to: self.fee_vault.to_account_info(),
            authority: self.buyer.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
    fn escrow_to_buyer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.escrow_token_account.to_account_info(),
            to: self.buyer_receive_account.to_account_info(),
            authority: self.listing.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
    fn close_escrow_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.escrow_token_account.to_account_info(),
            destination: self.seller_receiving_account.to_account_info(),
            authority: self.listing.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
    fn listing_seeds(&self) -> [&[u8]; 5] {
        [
            b"listing",
            self.listing.seller.as_ref(),
            self.listing.mint.as_ref(),
            &self.listing.nonce.to_le_bytes(),
            &[self.listing.bump],
        ]
    }
}

#[derive(Accounts)]
pub struct Cancel<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub listing: Account<'info, Listing>,
    /// CHECK: seller of the listing
    #[account(address = listing.seller)]
    pub seller: AccountInfo<'info>,
    #[account(
        mut,
        constraint = seller_token_account.owner == seller.key(),
        constraint = seller_token_account.mint == listing.mint
    )]
    pub seller_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = listing.mint,
        associated_token::authority = listing
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Cancel<'info> {
    fn escrow_to_seller_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.escrow_token_account.to_account_info(),
            to: self.seller_token_account.to_account_info(),
            authority: self.listing.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
    fn close_escrow_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.escrow_token_account.to_account_info(),
            destination: self.seller.to_account_info(),
            authority: self.listing.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
    fn listing_seeds(&self) -> [&[u8]; 5] {
        [
            b"listing",
            self.listing.seller.as_ref(),
            self.listing.mint.as_ref(),
            &self.listing.nonce.to_le_bytes(),
            &[self.listing.bump],
        ]
    }
}

#[derive(Accounts)]
pub struct WithdrawFee<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        constraint = admin_fee_account.owner == admin.key(),
        constraint = admin_fee_account.mint == fee_vault.mint
    )]
    pub admin_fee_account: Account<'info, TokenAccount>,
    #[account(mut, seeds=[b"fee_vault"], bump)]
    pub fee_vault: Account<'info, TokenAccount>,
    /// CHECK: PDA authority for fee vault
    #[account(seeds=[b"fee_vault"], bump)]
    pub fee_vault_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> WithdrawFee<'info> {
    fn vault_to_admin_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.fee_vault.to_account_info(),
            to: self.admin_fee_account.to_account_info(),
            authority: self.fee_vault_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[account]
pub struct Listing {
    pub seller: Pubkey,
    pub mint: Pubkey,
    pub price: u64,
    pub quantity: u64,
    pub expiry: i64,
    pub status: u8,
    pub bump: u8,
    pub nonce: u64,
}

impl Listing {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 8 + 1 + 1 + 8;
}

#[repr(u8)]
pub enum ListingStatus {
    Open = 0,
    Settled = 1,
    Canceled = 2,
}

#[event]
pub struct ListingCreated {
    pub seller: Pubkey,
    pub listing: Pubkey,
    pub mint: Pubkey,
    pub price: u64,
    pub quantity: u64,
}

#[event]
pub struct Purchased {
    pub listing: Pubkey,
    pub buyer: Pubkey,
    pub price: u64,
}

#[event]
pub struct Canceled {
    pub listing: Pubkey,
    pub seller: Pubkey,
}

#[event]
pub struct FeeWithdrawn {
    pub admin: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum EscrowError {
    #[msg("Listing is not in correct state")]
    InvalidState,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Overflow")]
    Overflow,
    #[msg("Invalid quantity")]
    InvalidQuantity,
    #[msg("Invalid price")]
    InvalidPrice,
}
