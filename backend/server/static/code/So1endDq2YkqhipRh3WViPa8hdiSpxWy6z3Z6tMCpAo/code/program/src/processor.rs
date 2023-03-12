//! Program state processor

use crate::{
    self as solend_program,
    error::LendingError,
    instruction::LendingInstruction,
    math::{Decimal, Rate, TryAdd, TryDiv, TryMul, TrySub, WAD},
    oracles::get_pyth_price,
    state::{
        CalculateBorrowResult, CalculateLiquidationResult, CalculateRepayResult,
        InitLendingMarketParams, InitObligationParams, InitReserveParams, LendingMarket,
        NewReserveCollateralParams, NewReserveLiquidityParams, Obligation, Reserve,
        ReserveCollateral, ReserveConfig, ReserveLiquidity,
    },
};
use pyth_sdk_solana::{self, state::ProductAccount};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{get_stack_height, Instruction, TRANSACTION_LEVEL_STACK_HEIGHT},
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::instructions::{load_current_index_checked, load_instruction_at_checked},
    sysvar::{
        clock::{self, Clock},
        rent::Rent,
        Sysvar,
    },
};
use solend_sdk::{switchboard_v2_devnet, switchboard_v2_mainnet};
use spl_token::state::Mint;
use std::{cmp::min, result::Result};
use switchboard_program::{
    get_aggregator, get_aggregator_result, AggregatorState, RoundResult, SwitchboardAccountType,
};
use switchboard_v2::AggregatorAccountData;

/// Processes an instruction
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = LendingInstruction::unpack(input)?;
    match instruction {
        LendingInstruction::InitLendingMarket {
            owner,
            quote_currency,
        } => {
            msg!("Instruction: Init Lending Market");
            process_init_lending_market(program_id, owner, quote_currency, accounts)
        }
        LendingInstruction::SetLendingMarketOwner { new_owner } => {
            msg!("Instruction: Set Lending Market Owner");
            process_set_lending_market_owner(program_id, new_owner, accounts)
        }
        LendingInstruction::InitReserve {
            liquidity_amount,
            config,
        } => {
            msg!("Instruction: Init Reserve");
            process_init_reserve(program_id, liquidity_amount, config, accounts)
        }
        LendingInstruction::RefreshReserve => {
            msg!("Instruction: Refresh Reserve");
            process_refresh_reserve(program_id, accounts)
        }
        LendingInstruction::DepositReserveLiquidity { liquidity_amount } => {
            msg!("Instruction: Deposit Reserve Liquidity");
            process_deposit_reserve_liquidity(program_id, liquidity_amount, accounts)
        }
        LendingInstruction::RedeemReserveCollateral { collateral_amount } => {
            msg!("Instruction: Redeem Reserve Collateral");
            process_redeem_reserve_collateral(program_id, collateral_amount, accounts)
        }
        LendingInstruction::InitObligation => {
            msg!("Instruction: Init Obligation");
            process_init_obligation(program_id, accounts)
        }
        LendingInstruction::RefreshObligation => {
            msg!("Instruction: Refresh Obligation");
            process_refresh_obligation(program_id, accounts)
        }
        LendingInstruction::DepositObligationCollateral { collateral_amount } => {
            msg!("Instruction: Deposit Obligation Collateral");
            process_deposit_obligation_collateral(program_id, collateral_amount, accounts)
        }
        LendingInstruction::WithdrawObligationCollateral { collateral_amount } => {
            msg!("Instruction: Withdraw Obligation Collateral");
            process_withdraw_obligation_collateral(program_id, collateral_amount, accounts)
        }
        LendingInstruction::BorrowObligationLiquidity { liquidity_amount } => {
            msg!("Instruction: Borrow Obligation Liquidity");
            process_borrow_obligation_liquidity(program_id, liquidity_amount, accounts)
        }
        LendingInstruction::RepayObligationLiquidity { liquidity_amount } => {
            msg!("Instruction: Repay Obligation Liquidity");
            process_repay_obligation_liquidity(program_id, liquidity_amount, accounts)
        }
        LendingInstruction::LiquidateObligation { .. } => {
            msg!("Instruction: Liquidate Obligation");
            msg!("method deprecated, please migrate to Liquidate Obligation and Redeem Reserve Collateral");
            Err(LendingError::DeprecatedInstruction.into())
        }
        LendingInstruction::FlashLoan { .. } => {
            msg!("Instruction: Flash Loan");
            msg!("This instruction has been deprecated. Use FlashBorrowReserveLiquidity instead");
            Err(LendingError::DeprecatedInstruction.into())
        }
        LendingInstruction::DepositReserveLiquidityAndObligationCollateral { liquidity_amount } => {
            msg!("Instruction: Deposit Reserve Liquidity and Obligation Collateral");
            process_deposit_reserve_liquidity_and_obligation_collateral(
                program_id,
                liquidity_amount,
                accounts,
            )
        }
        LendingInstruction::WithdrawObligationCollateralAndRedeemReserveCollateral {
            collateral_amount,
        } => {
            msg!("Instruction: Withdraw Obligation Collateral and Redeem Reserve Collateral");
            process_withdraw_obligation_collateral_and_redeem_reserve_liquidity(
                program_id,
                collateral_amount,
                accounts,
            )
        }
        LendingInstruction::UpdateReserveConfig { config } => {
            msg!("Instruction: UpdateReserveConfig");
            process_update_reserve_config(program_id, config, accounts)
        }
        LendingInstruction::LiquidateObligationAndRedeemReserveCollateral { liquidity_amount } => {
            msg!("Instruction: Liquidate Obligation and Redeem Reserve Collateral");
            process_liquidate_obligation_and_redeem_reserve_collateral(
                program_id,
                liquidity_amount,
                accounts,
            )
        }
        LendingInstruction::RedeemFees => {
            msg!("Instruction: RedeemFees");
            process_redeem_fees(program_id, accounts)
        }
        LendingInstruction::FlashBorrowReserveLiquidity { liquidity_amount } => {
            msg!("Instruction: Flash Borrow Reserve Liquidity");
            process_flash_borrow_reserve_liquidity(program_id, liquidity_amount, accounts)
        }
        LendingInstruction::FlashRepayReserveLiquidity {
            liquidity_amount,
            borrow_instruction_index,
        } => {
            msg!("Instruction: Flash Repay Reserve Liquidity");
            process_flash_repay_reserve_liquidity(
                program_id,
                liquidity_amount,
                borrow_instruction_index,
                accounts,
            )
        }
    }
}

fn process_init_lending_market(
    program_id: &Pubkey,
    owner: Pubkey,
    quote_currency: [u8; 32],
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let lending_market_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    let token_program_id = next_account_info(account_info_iter)?;
    let oracle_program_id = next_account_info(account_info_iter)?;
    let switchboard_oracle_program_id = next_account_info(account_info_iter)?;

    assert_rent_exempt(rent, lending_market_info)?;
    let mut lending_market = assert_uninitialized::<LendingMarket>(lending_market_info)?;
    if lending_market_info.owner != program_id {
        msg!("Lending market provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }

    lending_market.init(InitLendingMarketParams {
        bump_seed: Pubkey::find_program_address(&[lending_market_info.key.as_ref()], program_id).1,
        owner,
        quote_currency,
        token_program_id: *token_program_id.key,
        oracle_program_id: *oracle_program_id.key,
        switchboard_oracle_program_id: *switchboard_oracle_program_id.key,
    });
    LendingMarket::pack(lending_market, &mut lending_market_info.data.borrow_mut())?;

    Ok(())
}

#[inline(never)] // avoid stack frame limit
fn process_set_lending_market_owner(
    program_id: &Pubkey,
    new_owner: Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let lending_market_info = next_account_info(account_info_iter)?;
    let lending_market_owner_info = next_account_info(account_info_iter)?;

    let mut lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        msg!("Lending market provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.owner != lending_market_owner_info.key {
        msg!("Lending market owner does not match the lending market owner provided");
        return Err(LendingError::InvalidMarketOwner.into());
    }
    if !lending_market_owner_info.is_signer {
        msg!("Lending market owner provided must be a signer");
        return Err(LendingError::InvalidSigner.into());
    }

    lending_market.owner = new_owner;
    LendingMarket::pack(lending_market, &mut lending_market_info.data.borrow_mut())?;

    Ok(())
}

fn process_init_reserve(
    program_id: &Pubkey,
    liquidity_amount: u64,
    config: ReserveConfig,
    accounts: &[AccountInfo],
) -> ProgramResult {
    if liquidity_amount == 0 {
        msg!("Reserve must be initialized with liquidity");
        return Err(LendingError::InvalidAmount.into());
    }
    validate_reserve_config(config)?;
    let account_info_iter = &mut accounts.iter().peekable();
    let source_liquidity_info = next_account_info(account_info_iter)?;
    let destination_collateral_info = next_account_info(account_info_iter)?;
    let reserve_info = next_account_info(account_info_iter)?;
    let reserve_liquidity_mint_info = next_account_info(account_info_iter)?;
    let reserve_liquidity_supply_info = next_account_info(account_info_iter)?;
    let reserve_liquidity_fee_receiver_info = next_account_info(account_info_iter)?;
    let reserve_collateral_mint_info = next_account_info(account_info_iter)?;
    let reserve_collateral_supply_info = next_account_info(account_info_iter)?;
    let pyth_product_info = next_account_info(account_info_iter)?;
    let pyth_price_info = next_account_info(account_info_iter)?;
    let switchboard_feed_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let lending_market_authority_info = next_account_info(account_info_iter)?;
    let lending_market_owner_info = next_account_info(account_info_iter)?;
    let user_transfer_authority_info = next_account_info(account_info_iter)?;

    let clock = &Clock::get()?;
    if account_info_iter.peek().map(|a| a.key) == Some(&clock::ID) {
        next_account_info(account_info_iter)?;
    }

    let rent_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(rent_info)?;
    let token_program_id = next_account_info(account_info_iter)?;

    assert_rent_exempt(rent, reserve_info)?;
    let mut reserve = assert_uninitialized::<Reserve>(reserve_info)?;
    if reserve_info.owner != program_id {
        msg!(
            "Reserve provided is not owned by the lending program {} != {}",
            &reserve_info.owner.to_string(),
            &program_id.to_string(),
        );
        return Err(LendingError::InvalidAccountOwner.into());
    }

    if reserve_liquidity_supply_info.key == source_liquidity_info.key {
        msg!("Reserve liquidity supply cannot be used as the source liquidity provided");
        return Err(LendingError::InvalidAccountInput.into());
    }

    let lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        msg!(
            "Lending market provided is not owned by the lending program  {} != {}",
            &lending_market_info.owner.to_string(),
            &program_id.to_string(),
        );
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.token_program_id != token_program_id.key {
        msg!("Lending market token program does not match the token program provided");
        return Err(LendingError::InvalidTokenProgram.into());
    }
    if &lending_market.owner != lending_market_owner_info.key {
        msg!("Lending market owner does not match the lending market owner provided");
        return Err(LendingError::InvalidMarketOwner.into());
    }
    if !lending_market_owner_info.is_signer {
        msg!("Lending market owner provided must be a signer");
        return Err(LendingError::InvalidSigner.into());
    }
    if *switchboard_feed_info.key == solend_program::NULL_PUBKEY
        && (*pyth_price_info.key == solend_program::NULL_PUBKEY
            || *pyth_product_info.key == solend_program::NULL_PUBKEY)
    {
        msg!("Both price oracles are null. At least one must be non-null");
        return Err(LendingError::InvalidOracleConfig.into());
    }
    validate_pyth_keys(&lending_market, pyth_product_info, pyth_price_info)?;
    validate_switchboard_keys(&lending_market, switchboard_feed_info)?;

    let market_price = get_price(Some(switchboard_feed_info), pyth_price_info, clock)?;

    let authority_signer_seeds = &[
        lending_market_info.key.as_ref(),
        &[lending_market.bump_seed],
    ];
    let lending_market_authority_pubkey =
        Pubkey::create_program_address(authority_signer_seeds, program_id)?;
    if &lending_market_authority_pubkey != lending_market_authority_info.key {
        msg!(
            "Derived lending market authority does not match the lending market authority provided"
        );
        return Err(LendingError::InvalidMarketAuthority.into());
    }

    let reserve_liquidity_mint = unpack_mint(&reserve_liquidity_mint_info.data.borrow())?;
    if reserve_liquidity_mint_info.owner != token_program_id.key {
        msg!("Reserve liquidity mint is not owned by the token program provided");
        return Err(LendingError::InvalidTokenOwner.into());
    }

    reserve.init(InitReserveParams {
        current_slot: clock.slot,
        lending_market: *lending_market_info.key,
        liquidity: ReserveLiquidity::new(NewReserveLiquidityParams {
            mint_pubkey: *reserve_liquidity_mint_info.key,
            mint_decimals: reserve_liquidity_mint.decimals,
            supply_pubkey: *reserve_liquidity_supply_info.key,
            pyth_oracle_pubkey: *pyth_price_info.key,
            switchboard_oracle_pubkey: *switchboard_feed_info.key,
            market_price,
        }),
        collateral: ReserveCollateral::new(NewReserveCollateralParams {
            mint_pubkey: *reserve_collateral_mint_info.key,
            supply_pubkey: *reserve_collateral_supply_info.key,
        }),
        config,
    });

    let collateral_amount = reserve.deposit_liquidity(liquidity_amount)?;
    Reserve::pack(reserve, &mut reserve_info.data.borrow_mut())?;

    spl_token_init_account(TokenInitializeAccountParams {
        account: reserve_liquidity_supply_info.clone(),
        mint: reserve_liquidity_mint_info.clone(),
        owner: lending_market_authority_info.clone(),
        rent: rent_info.clone(),
        token_program: token_program_id.clone(),
    })?;

    spl_token_init_account(TokenInitializeAccountParams {
        account: reserve_liquidity_fee_receiver_info.clone(),
        mint: reserve_liquidity_mint_info.clone(),
        owner: lending_market_authority_info.clone(),
        rent: rent_info.clone(),
        token_program: token_program_id.clone(),
    })?;

    spl_token_init_mint(TokenInitializeMintParams {
        mint: reserve_collateral_mint_info.clone(),
        authority: lending_market_authority_info.key,
        rent: rent_info.clone(),
        decimals: reserve_liquidity_mint.decimals,
        token_program: token_program_id.clone(),
    })?;

    spl_token_init_account(TokenInitializeAccountParams {
        account: reserve_collateral_supply_info.clone(),
        mint: reserve_collateral_mint_info.clone(),
        owner: lending_market_authority_info.clone(),
        rent: rent_info.clone(),
        token_program: token_program_id.clone(),
    })?;

    spl_token_init_account(TokenInitializeAccountParams {
        account: destination_collateral_info.clone(),
        mint: reserve_collateral_mint_info.clone(),
        owner: user_transfer_authority_info.clone(),
        rent: rent_info.clone(),
        token_program: token_program_id.clone(),
    })?;

    spl_token_transfer(TokenTransferParams {
        source: source_liquidity_info.clone(),
        destination: reserve_liquidity_supply_info.clone(),
        amount: liquidity_amount,
        authority: user_transfer_authority_info.clone(),
        authority_signer_seeds: &[],
        token_program: token_program_id.clone(),
    })?;

    spl_token_mint_to(TokenMintToParams {
        mint: reserve_collateral_mint_info.clone(),
        destination: destination_collateral_info.clone(),
        amount: collateral_amount,
        authority: lending_market_authority_info.clone(),
        authority_signer_seeds,
        token_program: token_program_id.clone(),
    })?;

    Ok(())
}

fn process_refresh_reserve(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().peekable();
    let reserve_info = next_account_info(account_info_iter)?;
    let pyth_price_info = next_account_info(account_info_iter)?;
    // set switchboard to a placeholder account info
    let mut switchboard_feed_info = None;
    // if the next account info exists and is not the clock set it to be switchboard
    let switchboard_peek = account_info_iter.peek().map(|a| a.key);
    if switchboard_peek.is_some() && switchboard_peek != Some(&clock::ID) {
        switchboard_feed_info = Some(next_account_info(account_info_iter)?);
    }
    let clock = &Clock::get()?;
    if account_info_iter.peek().map(|a| a.key) == Some(&clock::ID) {
        next_account_info(account_info_iter)?;
    }
    _refresh_reserve(
        program_id,
        reserve_info,
        pyth_price_info,
        switchboard_feed_info,
        clock,
    )
}

fn _refresh_reserve<'a>(
    program_id: &Pubkey,
    reserve_info: &AccountInfo<'a>,
    pyth_price_info: &AccountInfo<'a>,
    switchboard_feed_info: Option<&AccountInfo<'a>>,
    clock: &Clock,
) -> ProgramResult {
    let mut reserve = Reserve::unpack(&reserve_info.data.borrow())?;
    if reserve_info.owner != program_id {
        msg!("Reserve provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &reserve.liquidity.pyth_oracle_pubkey != pyth_price_info.key {
        msg!("Reserve liquidity pyth oracle does not match the reserve liquidity pyth oracle provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    // the first check is to allow for the only passing in pyth case
    // TODO maybe change this to is_some_and later
    if switchboard_feed_info.is_some()
        && &reserve.liquidity.switchboard_oracle_pubkey != switchboard_feed_info.unwrap().key
    {
        msg!("Reserve liquidity switchboard oracle does not match the reserve liquidity switchboard oracle provided");
        return Err(LendingError::InvalidOracleConfig.into());
    }

    reserve.liquidity.market_price = get_price(switchboard_feed_info, pyth_price_info, clock)?;
    Reserve::pack(reserve, &mut reserve_info.data.borrow_mut())?;

    _refresh_reserve_interest(program_id, reserve_info, clock)
}

/// Lite version of refresh_reserve that should be used when the oracle price doesn't need to be updated
/// BE CAREFUL WHEN USING THIS
fn _refresh_reserve_interest<'a>(
    program_id: &Pubkey,
    reserve_info: &AccountInfo<'a>,
    clock: &Clock,
) -> ProgramResult {
    let mut reserve = Reserve::unpack(&reserve_info.data.borrow())?;
    if reserve_info.owner != program_id {
        msg!("Reserve provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }

    reserve.accrue_interest(clock.slot)?;
    reserve.last_update.update_slot(clock.slot);
    Reserve::pack(reserve, &mut reserve_info.data.borrow_mut())?;

    Ok(())
}

fn process_deposit_reserve_liquidity(
    program_id: &Pubkey,
    liquidity_amount: u64,
    accounts: &[AccountInfo],
) -> ProgramResult {
    if liquidity_amount == 0 {
        msg!("Liquidity amount provided cannot be zero");
        return Err(LendingError::InvalidAmount.into());
    }

    let account_info_iter = &mut accounts.iter().peekable();
    let source_liquidity_info = next_account_info(account_info_iter)?;
    let destination_collateral_info = next_account_info(account_info_iter)?;
    let reserve_info = next_account_info(account_info_iter)?;
    let reserve_liquidity_supply_info = next_account_info(account_info_iter)?;
    let reserve_collateral_mint_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let lending_market_authority_info = next_account_info(account_info_iter)?;
    let user_transfer_authority_info = next_account_info(account_info_iter)?;
    let clock = &Clock::get()?;
    if account_info_iter.peek().map(|a| a.key) == Some(&clock::ID) {
        next_account_info(account_info_iter)?;
    }
    let token_program_id = next_account_info(account_info_iter)?;

    _refresh_reserve_interest(program_id, reserve_info, clock)?;
    _deposit_reserve_liquidity(
        program_id,
        liquidity_amount,
        source_liquidity_info,
        destination_collateral_info,
        reserve_info,
        reserve_liquidity_supply_info,
        reserve_collateral_mint_info,
        lending_market_info,
        lending_market_authority_info,
        user_transfer_authority_info,
        clock,
        token_program_id,
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _deposit_reserve_liquidity<'a>(
    program_id: &Pubkey,
    liquidity_amount: u64,
    source_liquidity_info: &AccountInfo<'a>,
    destination_collateral_info: &AccountInfo<'a>,
    reserve_info: &AccountInfo<'a>,
    reserve_liquidity_supply_info: &AccountInfo<'a>,
    reserve_collateral_mint_info: &AccountInfo<'a>,
    lending_market_info: &AccountInfo<'a>,
    lending_market_authority_info: &AccountInfo<'a>,
    user_transfer_authority_info: &AccountInfo<'a>,
    clock: &Clock,
    token_program_id: &AccountInfo<'a>,
) -> Result<u64, ProgramError> {
    let lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        msg!("Lending market provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.token_program_id != token_program_id.key {
        msg!("Lending market token program does not match the token program provided");
        return Err(LendingError::InvalidTokenProgram.into());
    }
    let mut reserve = Reserve::unpack(&reserve_info.data.borrow())?;
    if reserve_info.owner != program_id {
        msg!("Reserve provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &reserve.lending_market != lending_market_info.key {
        msg!("Reserve lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &reserve.liquidity.supply_pubkey != reserve_liquidity_supply_info.key {
        msg!("Reserve liquidity supply does not match the reserve liquidity supply provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &reserve.collateral.mint_pubkey != reserve_collateral_mint_info.key {
        msg!("Reserve collateral mint does not match the reserve collateral mint provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &reserve.liquidity.supply_pubkey == source_liquidity_info.key {
        msg!("Reserve liquidity supply cannot be used as the source liquidity provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &reserve.collateral.supply_pubkey == destination_collateral_info.key {
        msg!("Reserve collateral supply cannot be used as the destination collateral provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if reserve.last_update.is_stale(clock.slot)? {
        msg!("Reserve is stale and must be refreshed in the current slot");
        return Err(LendingError::ReserveStale.into());
    }
    let authority_signer_seeds = &[
        lending_market_info.key.as_ref(),
        &[lending_market.bump_seed],
    ];
    let lending_market_authority_pubkey =
        Pubkey::create_program_address(authority_signer_seeds, program_id)?;
    if &lending_market_authority_pubkey != lending_market_authority_info.key {
        msg!(
            "Derived lending market authority {} does not match the lending market authority provided {}",
            &lending_market_authority_pubkey.to_string(),
            &lending_market_authority_info.key.to_string(),
        );
        return Err(LendingError::InvalidMarketAuthority.into());
    }

    if Decimal::from(liquidity_amount)
        .try_add(reserve.liquidity.total_supply()?)?
        .try_floor_u64()?
        > reserve.config.deposit_limit
    {
        msg!("Cannot deposit liquidity above the reserve deposit limit");
        return Err(LendingError::InvalidAmount.into());
    }

    let collateral_amount = reserve.deposit_liquidity(liquidity_amount)?;
    reserve.last_update.mark_stale();
    Reserve::pack(reserve, &mut reserve_info.data.borrow_mut())?;

    spl_token_transfer(TokenTransferParams {
        source: source_liquidity_info.clone(),
        destination: reserve_liquidity_supply_info.clone(),
        amount: liquidity_amount,
        authority: user_transfer_authority_info.clone(),
        authority_signer_seeds: &[],
        token_program: token_program_id.clone(),
    })?;

    spl_token_mint_to(TokenMintToParams {
        mint: reserve_collateral_mint_info.clone(),
        destination: destination_collateral_info.clone(),
        amount: collateral_amount,
        authority: lending_market_authority_info.clone(),
        authority_signer_seeds,
        token_program: token_program_id.clone(),
    })?;

    Ok(collateral_amount)
}

fn process_redeem_reserve_collateral(
    program_id: &Pubkey,
    collateral_amount: u64,
    accounts: &[AccountInfo],
) -> ProgramResult {
    if collateral_amount == 0 {
        msg!("Collateral amount provided cannot be zero");
        return Err(LendingError::InvalidAmount.into());
    }

    let account_info_iter = &mut accounts.iter().peekable();
    let source_collateral_info = next_account_info(account_info_iter)?;
    let destination_liquidity_info = next_account_info(account_info_iter)?;
    let reserve_info = next_account_info(account_info_iter)?;
    let reserve_collateral_mint_info = next_account_info(account_info_iter)?;
    let reserve_liquidity_supply_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let lending_market_authority_info = next_account_info(account_info_iter)?;
    let user_transfer_authority_info = next_account_info(account_info_iter)?;
    let clock = &Clock::get()?;
    if account_info_iter.peek().map(|a| a.key) == Some(&clock::ID) {
        next_account_info(account_info_iter)?;
    }
    let token_program_id = next_account_info(account_info_iter)?;

    _refresh_reserve_interest(program_id, reserve_info, clock)?;
    _redeem_reserve_collateral(
        program_id,
        collateral_amount,
        source_collateral_info,
        destination_liquidity_info,
        reserve_info,
        reserve_collateral_mint_info,
        reserve_liquidity_supply_info,
        lending_market_info,
        lending_market_authority_info,
        user_transfer_authority_info,
        clock,
        token_program_id,
    )?;
    let mut reserve = Reserve::unpack(&reserve_info.data.borrow())?;
    reserve.last_update.mark_stale();
    Reserve::pack(reserve, &mut reserve_info.data.borrow_mut())?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _redeem_reserve_collateral<'a>(
    program_id: &Pubkey,
    collateral_amount: u64,
    source_collateral_info: &AccountInfo<'a>,
    destination_liquidity_info: &AccountInfo<'a>,
    reserve_info: &AccountInfo<'a>,
    reserve_collateral_mint_info: &AccountInfo<'a>,
    reserve_liquidity_supply_info: &AccountInfo<'a>,
    lending_market_info: &AccountInfo<'a>,
    lending_market_authority_info: &AccountInfo<'a>,
    user_transfer_authority_info: &AccountInfo<'a>,
    clock: &Clock,
    token_program_id: &AccountInfo<'a>,
) -> Result<u64, ProgramError> {
    let lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        msg!("Lending market provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.token_program_id != token_program_id.key {
        msg!("Lending market token program does not match the token program provided");
        return Err(LendingError::InvalidTokenProgram.into());
    }

    let mut reserve = Reserve::unpack(&reserve_info.data.borrow())?;
    if reserve_info.owner != program_id {
        msg!("Reserve provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &reserve.lending_market != lending_market_info.key {
        msg!("Reserve lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &reserve.collateral.mint_pubkey != reserve_collateral_mint_info.key {
        msg!("Reserve collateral mint does not match the reserve collateral mint provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &reserve.collateral.supply_pubkey == source_collateral_info.key {
        msg!("Reserve collateral supply cannot be used as the source collateral provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &reserve.liquidity.supply_pubkey != reserve_liquidity_supply_info.key {
        msg!("Reserve liquidity supply does not match the reserve liquidity supply provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &reserve.liquidity.supply_pubkey == destination_liquidity_info.key {
        msg!("Reserve liquidity supply cannot be used as the destination liquidity provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if reserve.last_update.is_stale(clock.slot)? {
        msg!("Reserve is stale and must be refreshed in the current slot");
        return Err(LendingError::ReserveStale.into());
    }

    let authority_signer_seeds = &[
        lending_market_info.key.as_ref(),
        &[lending_market.bump_seed],
    ];
    let lending_market_authority_pubkey =
        Pubkey::create_program_address(authority_signer_seeds, program_id)?;
    if &lending_market_authority_pubkey != lending_market_authority_info.key {
        msg!(
            "Derived lending market authority does not match the lending market authority provided"
        );
        return Err(LendingError::InvalidMarketAuthority.into());
    }

    let liquidity_amount = reserve.redeem_collateral(collateral_amount)?;
    reserve.last_update.mark_stale();
    Reserve::pack(reserve, &mut reserve_info.data.borrow_mut())?;

    spl_token_burn(TokenBurnParams {
        mint: reserve_collateral_mint_info.clone(),
        source: source_collateral_info.clone(),
        amount: collateral_amount,
        authority: user_transfer_authority_info.clone(),
        authority_signer_seeds: &[],
        token_program: token_program_id.clone(),
    })?;

    spl_token_transfer(TokenTransferParams {
        source: reserve_liquidity_supply_info.clone(),
        destination: destination_liquidity_info.clone(),
        amount: liquidity_amount,
        authority: lending_market_authority_info.clone(),
        authority_signer_seeds,
        token_program: token_program_id.clone(),
    })?;

    Ok(liquidity_amount)
}

#[inline(never)] // avoid stack frame limit
fn process_init_obligation(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().peekable();
    let obligation_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let obligation_owner_info = next_account_info(account_info_iter)?;
    let clock = &Clock::get()?;
    if account_info_iter.peek().map(|a| a.key) == Some(&clock::ID) {
        next_account_info(account_info_iter)?;
    }
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    let token_program_id = next_account_info(account_info_iter)?;

    assert_rent_exempt(rent, obligation_info)?;
    let mut obligation = assert_uninitialized::<Obligation>(obligation_info)?;
    if obligation_info.owner != program_id {
        msg!("Obligation provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }

    let lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        msg!("Lending market provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.token_program_id != token_program_id.key {
        msg!("Lending market token program does not match the token program provided");
        return Err(LendingError::InvalidTokenProgram.into());
    }

    if !obligation_owner_info.is_signer {
        msg!("Obligation owner provided must be a signer");
        return Err(LendingError::InvalidSigner.into());
    }

    obligation.init(InitObligationParams {
        current_slot: clock.slot,
        lending_market: *lending_market_info.key,
        owner: *obligation_owner_info.key,
        deposits: vec![],
        borrows: vec![],
    });
    Obligation::pack(obligation, &mut obligation_info.data.borrow_mut())?;

    Ok(())
}

fn process_refresh_obligation(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().peekable();
    let obligation_info = next_account_info(account_info_iter)?;
    let clock = &Clock::get()?;
    if account_info_iter.peek().map(|a| a.key) == Some(&clock::ID) {
        next_account_info(account_info_iter)?;
    }

    let mut obligation = Obligation::unpack(&obligation_info.data.borrow())?;
    if obligation_info.owner != program_id {
        msg!("Obligation provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }

    let mut deposited_value = Decimal::zero();
    let mut borrowed_value = Decimal::zero();
    let mut allowed_borrow_value = Decimal::zero();
    let mut unhealthy_borrow_value = Decimal::zero();

    for (index, collateral) in obligation.deposits.iter_mut().enumerate() {
        let deposit_reserve_info = next_account_info(account_info_iter)?;
        if deposit_reserve_info.owner != program_id {
            msg!(
                "Deposit reserve provided for collateral {} is not owned by the lending program",
                index
            );
            return Err(LendingError::InvalidAccountOwner.into());
        }
        if collateral.deposit_reserve != *deposit_reserve_info.key {
            msg!(
                "Deposit reserve of collateral {} does not match the deposit reserve provided",
                index
            );
            return Err(LendingError::InvalidAccountInput.into());
        }

        let deposit_reserve = Reserve::unpack(&deposit_reserve_info.data.borrow())?;
        if deposit_reserve.last_update.is_stale(clock.slot)? {
            msg!(
                "Deposit reserve provided for collateral {} is stale and must be refreshed in the current slot",
                index
            );
            return Err(LendingError::ReserveStale.into());
        }

        // @TODO: add lookup table https://git.io/JOCYq
        let decimals = 10u64
            .checked_pow(deposit_reserve.liquidity.mint_decimals as u32)
            .ok_or(LendingError::MathOverflow)?;

        let market_value = deposit_reserve
            .collateral_exchange_rate()?
            .decimal_collateral_to_liquidity(collateral.deposited_amount.into())?
            .try_mul(deposit_reserve.liquidity.market_price)?
            .try_div(decimals)?;
        collateral.market_value = market_value;

        let loan_to_value_rate = Rate::from_percent(deposit_reserve.config.loan_to_value_ratio);
        let liquidation_threshold_rate =
            Rate::from_percent(deposit_reserve.config.liquidation_threshold);

        deposited_value = deposited_value.try_add(market_value)?;
        allowed_borrow_value =
            allowed_borrow_value.try_add(market_value.try_mul(loan_to_value_rate)?)?;
        unhealthy_borrow_value =
            unhealthy_borrow_value.try_add(market_value.try_mul(liquidation_threshold_rate)?)?;
    }

    for (index, liquidity) in obligation.borrows.iter_mut().enumerate() {
        let borrow_reserve_info = next_account_info(account_info_iter)?;
        if borrow_reserve_info.owner != program_id {
            msg!(
                "Borrow reserve provided for liquidity {} is not owned by the lending program",
                index
            );
            return Err(LendingError::InvalidAccountOwner.into());
        }
        if liquidity.borrow_reserve != *borrow_reserve_info.key {
            msg!(
                "Borrow reserve of liquidity {} does not match the borrow reserve provided",
                index
            );
            return Err(LendingError::InvalidAccountInput.into());
        }

        let borrow_reserve = Reserve::unpack(&borrow_reserve_info.data.borrow())?;
        if borrow_reserve.last_update.is_stale(clock.slot)? {
            msg!(
                "Borrow reserve provided for liquidity {} is stale and must be refreshed in the current slot",
                index
            );
            return Err(LendingError::ReserveStale.into());
        }

        liquidity.accrue_interest(borrow_reserve.liquidity.cumulative_borrow_rate_wads)?;

        // @TODO: add lookup table https://git.io/JOCYq
        let decimals = 10u64
            .checked_pow(borrow_reserve.liquidity.mint_decimals as u32)
            .ok_or(LendingError::MathOverflow)?;

        let market_value = liquidity
            .borrowed_amount_wads
            .try_mul(borrow_reserve.liquidity.market_price)?
            .try_div(decimals)?;
        liquidity.market_value = market_value;

        borrowed_value = borrowed_value.try_add(market_value)?;
    }

    if account_info_iter.peek().is_some() {
        msg!("Too many obligation deposit or borrow reserves provided");
        return Err(LendingError::InvalidAccountInput.into());
    }

    obligation.deposited_value = deposited_value;
    obligation.borrowed_value = borrowed_value;

    let global_unhealthy_borrow_value = Decimal::from(70000000u64);
    let global_allowed_borrow_value = Decimal::from(65000000u64);

    obligation.allowed_borrow_value = min(allowed_borrow_value, global_allowed_borrow_value);
    obligation.unhealthy_borrow_value = min(unhealthy_borrow_value, global_unhealthy_borrow_value);

    obligation.last_update.update_slot(clock.slot);
    Obligation::pack(obligation, &mut obligation_info.data.borrow_mut())?;

    Ok(())
}

#[inline(never)] // avoid stack frame limit
fn process_deposit_obligation_collateral(
    program_id: &Pubkey,
    collateral_amount: u64,
    accounts: &[AccountInfo],
) -> ProgramResult {
    if collateral_amount == 0 {
        msg!("Collateral amount provided cannot be zero");
        return Err(LendingError::InvalidAmount.into());
    }

    let account_info_iter = &mut accounts.iter().peekable();
    let source_collateral_info = next_account_info(account_info_iter)?;
    let destination_collateral_info = next_account_info(account_info_iter)?;
    let deposit_reserve_info = next_account_info(account_info_iter)?;
    let obligation_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let obligation_owner_info = next_account_info(account_info_iter)?;
    let user_transfer_authority_info = next_account_info(account_info_iter)?;
    let clock = &Clock::get()?;
    if account_info_iter.peek().map(|a| a.key) == Some(&clock::ID) {
        next_account_info(account_info_iter)?;
    }
    let token_program_id = next_account_info(account_info_iter)?;
    _refresh_reserve_interest(program_id, deposit_reserve_info, clock)?;
    _deposit_obligation_collateral(
        program_id,
        collateral_amount,
        source_collateral_info,
        destination_collateral_info,
        deposit_reserve_info,
        obligation_info,
        lending_market_info,
        obligation_owner_info,
        user_transfer_authority_info,
        clock,
        token_program_id,
    )?;
    let mut reserve = Reserve::unpack(&deposit_reserve_info.data.borrow())?;
    reserve.last_update.mark_stale();
    Reserve::pack(reserve, &mut deposit_reserve_info.data.borrow_mut())?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _deposit_obligation_collateral<'a>(
    program_id: &Pubkey,
    collateral_amount: u64,
    source_collateral_info: &AccountInfo<'a>,
    destination_collateral_info: &AccountInfo<'a>,
    deposit_reserve_info: &AccountInfo<'a>,
    obligation_info: &AccountInfo<'a>,
    lending_market_info: &AccountInfo<'a>,
    obligation_owner_info: &AccountInfo<'a>,
    user_transfer_authority_info: &AccountInfo<'a>,
    clock: &Clock,
    token_program_id: &AccountInfo<'a>,
) -> ProgramResult {
    let lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        msg!("Lending market provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.token_program_id != token_program_id.key {
        msg!("Lending market token program does not match the token program provided");
        return Err(LendingError::InvalidTokenProgram.into());
    }

    let deposit_reserve = Reserve::unpack(&deposit_reserve_info.data.borrow())?;
    if deposit_reserve_info.owner != program_id {
        msg!("Deposit reserve provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &deposit_reserve.lending_market != lending_market_info.key {
        msg!("Deposit reserve lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &deposit_reserve.collateral.supply_pubkey == source_collateral_info.key {
        msg!("Deposit reserve collateral supply cannot be used as the source collateral provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &deposit_reserve.collateral.supply_pubkey != destination_collateral_info.key {
        msg!(
            "Deposit reserve collateral supply must be used as the destination collateral provided"
        );
        return Err(LendingError::InvalidAccountInput.into());
    }
    if deposit_reserve.last_update.is_stale(clock.slot)? {
        msg!("Deposit reserve is stale and must be refreshed in the current slot");
        return Err(LendingError::ReserveStale.into());
    }

    let mut obligation = Obligation::unpack(&obligation_info.data.borrow())?;
    if obligation_info.owner != program_id {
        msg!("Obligation provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &obligation.lending_market != lending_market_info.key {
        msg!("Obligation lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &obligation.owner != obligation_owner_info.key {
        msg!("Obligation owner does not match the obligation owner provided");
        return Err(LendingError::InvalidObligationOwner.into());
    }
    if !obligation_owner_info.is_signer {
        msg!("Obligation owner provided must be a signer");
        return Err(LendingError::InvalidSigner.into());
    }

    obligation
        .find_or_add_collateral_to_deposits(*deposit_reserve_info.key)?
        .deposit(collateral_amount)?;
    obligation.last_update.mark_stale();
    Obligation::pack(obligation, &mut obligation_info.data.borrow_mut())?;
    spl_token_transfer(TokenTransferParams {
        source: source_collateral_info.clone(),
        destination: destination_collateral_info.clone(),
        amount: collateral_amount,
        authority: user_transfer_authority_info.clone(),
        authority_signer_seeds: &[],
        token_program: token_program_id.clone(),
    })?;
    Ok(())
}

#[inline(never)] // avoid stack frame limit
fn process_deposit_reserve_liquidity_and_obligation_collateral(
    program_id: &Pubkey,
    liquidity_amount: u64,
    accounts: &[AccountInfo],
) -> ProgramResult {
    if liquidity_amount == 0 {
        msg!("Liquidity amount provided cannot be zero");
        return Err(LendingError::InvalidAmount.into());
    }

    let account_info_iter = &mut accounts.iter().peekable();
    let source_liquidity_info = next_account_info(account_info_iter)?;
    let user_collateral_info = next_account_info(account_info_iter)?;
    let reserve_info = next_account_info(account_info_iter)?;
    let reserve_liquidity_supply_info = next_account_info(account_info_iter)?;
    let reserve_collateral_mint_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let lending_market_authority_info = next_account_info(account_info_iter)?;
    let destination_collateral_info = next_account_info(account_info_iter)?;
    let obligation_info = next_account_info(account_info_iter)?;
    let obligation_owner_info = next_account_info(account_info_iter)?;
    let _pyth_price_info = next_account_info(account_info_iter)?;
    let _switchboard_feed_info = next_account_info(account_info_iter)?;
    let user_transfer_authority_info = next_account_info(account_info_iter)?;
    let clock = &Clock::get()?;
    if account_info_iter.peek().map(|a| a.key) == Some(&clock::ID) {
        next_account_info(account_info_iter)?;
    }
    let token_program_id = next_account_info(account_info_iter)?;

    _refresh_reserve_interest(program_id, reserve_info, clock)?;
    let collateral_amount = _deposit_reserve_liquidity(
        program_id,
        liquidity_amount,
        source_liquidity_info,
        user_collateral_info,
        reserve_info,
        reserve_liquidity_supply_info,
        reserve_collateral_mint_info,
        lending_market_info,
        lending_market_authority_info,
        user_transfer_authority_info,
        clock,
        token_program_id,
    )?;
    _refresh_reserve_interest(program_id, reserve_info, clock)?;
    _deposit_obligation_collateral(
        program_id,
        collateral_amount,
        user_collateral_info,
        destination_collateral_info,
        reserve_info,
        obligation_info,
        lending_market_info,
        obligation_owner_info,
        user_transfer_authority_info,
        clock,
        token_program_id,
    )?;
    // mark the reserve as stale to make sure no weird bugs happen
    let mut reserve = Reserve::unpack(&reserve_info.data.borrow())?;
    reserve.last_update.mark_stale();
    Reserve::pack(reserve, &mut reserve_info.data.borrow_mut())?;

    Ok(())
}

#[inline(never)] // avoid stack frame limit
fn process_withdraw_obligation_collateral(
    program_id: &Pubkey,
    collateral_amount: u64,
    accounts: &[AccountInfo],
) -> ProgramResult {
    if collateral_amount == 0 {
        msg!("Collateral amount provided cannot be zero");
        return Err(LendingError::InvalidAmount.into());
    }

    let account_info_iter = &mut accounts.iter().peekable();
    let source_collateral_info = next_account_info(account_info_iter)?;
    let destination_collateral_info = next_account_info(account_info_iter)?;
    let withdraw_reserve_info = next_account_info(account_info_iter)?;
    let obligation_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let lending_market_authority_info = next_account_info(account_info_iter)?;
    let obligation_owner_info = next_account_info(account_info_iter)?;
    let clock = &Clock::get()?;
    if account_info_iter.peek().map(|a| a.key) == Some(&clock::ID) {
        next_account_info(account_info_iter)?;
    }
    let token_program_id = next_account_info(account_info_iter)?;
    _withdraw_obligation_collateral(
        program_id,
        collateral_amount,
        source_collateral_info,
        destination_collateral_info,
        withdraw_reserve_info,
        obligation_info,
        lending_market_info,
        lending_market_authority_info,
        obligation_owner_info,
        clock,
        token_program_id,
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _withdraw_obligation_collateral<'a>(
    program_id: &Pubkey,
    collateral_amount: u64,
    source_collateral_info: &AccountInfo<'a>,
    destination_collateral_info: &AccountInfo<'a>,
    withdraw_reserve_info: &AccountInfo<'a>,
    obligation_info: &AccountInfo<'a>,
    lending_market_info: &AccountInfo<'a>,
    lending_market_authority_info: &AccountInfo<'a>,
    obligation_owner_info: &AccountInfo<'a>,
    clock: &Clock,
    token_program_id: &AccountInfo<'a>,
) -> Result<u64, ProgramError> {
    let lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        msg!("Lending market provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.token_program_id != token_program_id.key {
        msg!("Lending market token program does not match the token program provided");
        return Err(LendingError::InvalidTokenProgram.into());
    }

    let withdraw_reserve = Reserve::unpack(&withdraw_reserve_info.data.borrow())?;
    if withdraw_reserve_info.owner != program_id {
        msg!("Withdraw reserve provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &withdraw_reserve.lending_market != lending_market_info.key {
        msg!("Withdraw reserve lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &withdraw_reserve.collateral.supply_pubkey != source_collateral_info.key {
        msg!("Withdraw reserve collateral supply must be used as the source collateral provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &withdraw_reserve.collateral.supply_pubkey == destination_collateral_info.key {
        msg!("Withdraw reserve collateral supply cannot be used as the destination collateral provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if withdraw_reserve.last_update.is_stale(clock.slot)? {
        msg!("Withdraw reserve is stale and must be refreshed in the current slot");
        return Err(LendingError::ReserveStale.into());
    }

    let mut obligation = Obligation::unpack(&obligation_info.data.borrow())?;
    if obligation_info.owner != program_id {
        msg!("Obligation provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &obligation.lending_market != lending_market_info.key {
        msg!("Obligation lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &obligation.owner != obligation_owner_info.key {
        msg!("Obligation owner does not match the obligation owner provided");
        return Err(LendingError::InvalidObligationOwner.into());
    }
    if !obligation_owner_info.is_signer {
        msg!("Obligation owner provided must be a signer");
        return Err(LendingError::InvalidSigner.into());
    }
    if obligation.last_update.is_stale(clock.slot)? {
        msg!("Obligation is stale and must be refreshed in the current slot");
        return Err(LendingError::ObligationStale.into());
    }

    let (collateral, collateral_index) =
        obligation.find_collateral_in_deposits(*withdraw_reserve_info.key)?;
    if collateral.deposited_amount == 0 {
        msg!("Collateral deposited amount is zero");
        return Err(LendingError::ObligationCollateralEmpty.into());
    }

    let authority_signer_seeds = &[
        lending_market_info.key.as_ref(),
        &[lending_market.bump_seed],
    ];
    let lending_market_authority_pubkey =
        Pubkey::create_program_address(authority_signer_seeds, program_id)?;
    if &lending_market_authority_pubkey != lending_market_authority_info.key {
        msg!(
            "Derived lending market authority does not match the lending market authority provided"
        );
        return Err(LendingError::InvalidMarketAuthority.into());
    }

    let withdraw_amount = if obligation.borrows.is_empty() {
        if collateral_amount == u64::MAX {
            collateral.deposited_amount
        } else {
            collateral.deposited_amount.min(collateral_amount)
        }
    } else if obligation.deposited_value == Decimal::zero() {
        msg!("Obligation deposited value is zero");
        return Err(LendingError::ObligationDepositsZero.into());
    } else {
        let max_withdraw_value = obligation.max_withdraw_value(Rate::from_percent(
            withdraw_reserve.config.loan_to_value_ratio,
        ))?;

        if max_withdraw_value == Decimal::zero() {
            msg!("Maximum withdraw value is zero");
            return Err(LendingError::WithdrawTooLarge.into());
        }

        let withdraw_amount = if collateral_amount == u64::MAX {
            let withdraw_value = max_withdraw_value.min(collateral.market_value);
            let withdraw_pct = withdraw_value.try_div(collateral.market_value)?;
            withdraw_pct
                .try_mul(collateral.deposited_amount)?
                .try_floor_u64()?
                .min(collateral.deposited_amount)
        } else {
            let withdraw_amount = collateral_amount.min(collateral.deposited_amount);
            let withdraw_pct =
                Decimal::from(withdraw_amount).try_div(collateral.deposited_amount)?;
            let withdraw_value = collateral.market_value.try_mul(withdraw_pct)?;
            if withdraw_value > max_withdraw_value {
                msg!("Withdraw value cannot exceed maximum withdraw value");
                return Err(LendingError::WithdrawTooLarge.into());
            }
            withdraw_amount
        };
        if withdraw_amount == 0 {
            msg!("Withdraw amount is too small to transfer collateral");
            return Err(LendingError::WithdrawTooSmall.into());
        }
        withdraw_amount
    };

    obligation.withdraw(withdraw_amount, collateral_index)?;
    obligation.last_update.mark_stale();
    Obligation::pack(obligation, &mut obligation_info.data.borrow_mut())?;

    spl_token_transfer(TokenTransferParams {
        source: source_collateral_info.clone(),
        destination: destination_collateral_info.clone(),
        amount: withdraw_amount,
        authority: lending_market_authority_info.clone(),
        authority_signer_seeds,
        token_program: token_program_id.clone(),
    })?;

    Ok(withdraw_amount)
}

#[inline(never)] // avoid stack frame limit
fn process_borrow_obligation_liquidity(
    program_id: &Pubkey,
    liquidity_amount: u64,
    accounts: &[AccountInfo],
) -> ProgramResult {
    if liquidity_amount == 0 {
        msg!("Liquidity amount provided cannot be zero");
        return Err(LendingError::InvalidAmount.into());
    }

    let account_info_iter = &mut accounts.iter().peekable();
    let source_liquidity_info = next_account_info(account_info_iter)?;
    let destination_liquidity_info = next_account_info(account_info_iter)?;
    let borrow_reserve_info = next_account_info(account_info_iter)?;
    let borrow_reserve_liquidity_fee_receiver_info = next_account_info(account_info_iter)?;
    let obligation_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let lending_market_authority_info = next_account_info(account_info_iter)?;
    let obligation_owner_info = next_account_info(account_info_iter)?;
    let clock = &Clock::get()?;
    if account_info_iter.peek().map(|a| a.key) == Some(&clock::ID) {
        next_account_info(account_info_iter)?;
    }
    let token_program_id = next_account_info(account_info_iter)?;

    let lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        msg!("Lending market provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.token_program_id != token_program_id.key {
        msg!("Lending market token program does not match the token program provided");
        return Err(LendingError::InvalidTokenProgram.into());
    }

    let mut borrow_reserve = Reserve::unpack(&borrow_reserve_info.data.borrow())?;
    if borrow_reserve_info.owner != program_id {
        msg!("Borrow reserve provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &borrow_reserve.lending_market != lending_market_info.key {
        msg!("Borrow reserve lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &borrow_reserve.liquidity.supply_pubkey != source_liquidity_info.key {
        msg!("Borrow reserve liquidity supply must be used as the source liquidity provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &borrow_reserve.liquidity.supply_pubkey == destination_liquidity_info.key {
        msg!(
            "Borrow reserve liquidity supply cannot be used as the destination liquidity provided"
        );
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &borrow_reserve.config.fee_receiver != borrow_reserve_liquidity_fee_receiver_info.key {
        msg!("Borrow reserve liquidity fee receiver does not match the borrow reserve liquidity fee receiver provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if borrow_reserve.last_update.is_stale(clock.slot)? {
        msg!("Borrow reserve is stale and must be refreshed in the current slot");
        return Err(LendingError::ReserveStale.into());
    }
    if liquidity_amount != u64::MAX
        && Decimal::from(liquidity_amount)
            .try_add(borrow_reserve.liquidity.borrowed_amount_wads)?
            .try_floor_u64()?
            > borrow_reserve.config.borrow_limit
    {
        msg!("Cannot borrow above the borrow limit");
        return Err(LendingError::InvalidAmount.into());
    }

    let mut obligation = Obligation::unpack(&obligation_info.data.borrow())?;
    if obligation_info.owner != program_id {
        msg!("Obligation provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &obligation.lending_market != lending_market_info.key {
        msg!("Obligation lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &obligation.owner != obligation_owner_info.key {
        msg!("Obligation owner does not match the obligation owner provided");
        return Err(LendingError::InvalidObligationOwner.into());
    }
    if !obligation_owner_info.is_signer {
        msg!("Obligation owner provided must be a signer");
        return Err(LendingError::InvalidSigner.into());
    }
    if obligation.last_update.is_stale(clock.slot)? {
        msg!("Obligation is stale and must be refreshed in the current slot");
        return Err(LendingError::ObligationStale.into());
    }
    if obligation.deposits.is_empty() {
        msg!("Obligation has no deposits to borrow against");
        return Err(LendingError::ObligationDepositsEmpty.into());
    }
    if obligation.deposited_value == Decimal::zero() {
        msg!("Obligation deposits have zero value");
        return Err(LendingError::ObligationDepositsZero.into());
    }

    let authority_signer_seeds = &[
        lending_market_info.key.as_ref(),
        &[lending_market.bump_seed],
    ];
    let lending_market_authority_pubkey =
        Pubkey::create_program_address(authority_signer_seeds, program_id)?;
    if &lending_market_authority_pubkey != lending_market_authority_info.key {
        msg!(
            "Derived lending market authority does not match the lending market authority provided"
        );
        return Err(LendingError::InvalidMarketAuthority.into());
    }

    let remaining_borrow_value = obligation.remaining_borrow_value()?;
    if remaining_borrow_value == Decimal::zero() {
        msg!("Remaining borrow value is zero");
        return Err(LendingError::BorrowTooLarge.into());
    }

    let remaining_reserve_capacity = Decimal::from(borrow_reserve.config.borrow_limit)
        .try_sub(borrow_reserve.liquidity.borrowed_amount_wads)
        .unwrap_or_else(|_| Decimal::zero());

    let CalculateBorrowResult {
        borrow_amount,
        receive_amount,
        borrow_fee,
        host_fee,
    } = borrow_reserve.calculate_borrow(
        liquidity_amount,
        remaining_borrow_value,
        remaining_reserve_capacity,
    )?;

    if receive_amount == 0 {
        msg!("Borrow amount is too small to receive liquidity after fees");
        return Err(LendingError::BorrowTooSmall.into());
    }

    let cumulative_borrow_rate_wads = borrow_reserve.liquidity.cumulative_borrow_rate_wads;

    borrow_reserve.liquidity.borrow(borrow_amount)?;
    borrow_reserve.last_update.mark_stale();
    Reserve::pack(borrow_reserve, &mut borrow_reserve_info.data.borrow_mut())?;

    let obligation_liquidity = obligation
        .find_or_add_liquidity_to_borrows(*borrow_reserve_info.key, cumulative_borrow_rate_wads)?;

    obligation_liquidity.borrow(borrow_amount)?;
    obligation.last_update.mark_stale();
    Obligation::pack(obligation, &mut obligation_info.data.borrow_mut())?;

    let mut owner_fee = borrow_fee;
    if let Ok(host_fee_receiver_info) = next_account_info(account_info_iter) {
        if host_fee > 0 {
            owner_fee = owner_fee
                .checked_sub(host_fee)
                .ok_or(LendingError::MathOverflow)?;

            spl_token_transfer(TokenTransferParams {
                source: source_liquidity_info.clone(),
                destination: host_fee_receiver_info.clone(),
                amount: host_fee,
                authority: lending_market_authority_info.clone(),
                authority_signer_seeds,
                token_program: token_program_id.clone(),
            })?;
        }
    }
    if owner_fee > 0 {
        spl_token_transfer(TokenTransferParams {
            source: source_liquidity_info.clone(),
            destination: borrow_reserve_liquidity_fee_receiver_info.clone(),
            amount: owner_fee,
            authority: lending_market_authority_info.clone(),
            authority_signer_seeds,
            token_program: token_program_id.clone(),
        })?;
    }

    spl_token_transfer(TokenTransferParams {
        source: source_liquidity_info.clone(),
        destination: destination_liquidity_info.clone(),
        amount: receive_amount,
        authority: lending_market_authority_info.clone(),
        authority_signer_seeds,
        token_program: token_program_id.clone(),
    })?;

    Ok(())
}

#[inline(never)] // avoid stack frame limit
fn process_repay_obligation_liquidity(
    program_id: &Pubkey,
    liquidity_amount: u64,
    accounts: &[AccountInfo],
) -> ProgramResult {
    if liquidity_amount == 0 {
        msg!("Liquidity amount provided cannot be zero");
        return Err(LendingError::InvalidAmount.into());
    }
    let account_info_iter = &mut accounts.iter().peekable();
    let source_liquidity_info = next_account_info(account_info_iter)?;
    let destination_liquidity_info = next_account_info(account_info_iter)?;
    let repay_reserve_info = next_account_info(account_info_iter)?;
    let obligation_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let user_transfer_authority_info = next_account_info(account_info_iter)?;
    let clock = &Clock::get()?;
    if account_info_iter.peek().map(|a| a.key) == Some(&clock::ID) {
        next_account_info(account_info_iter)?;
    }
    let token_program_id = next_account_info(account_info_iter)?;

    let lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        msg!("Lending market provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.token_program_id != token_program_id.key {
        msg!("Lending market token program does not match the token program provided");
        return Err(LendingError::InvalidTokenProgram.into());
    }

    _refresh_reserve_interest(program_id, repay_reserve_info, clock)?;
    let mut repay_reserve = Reserve::unpack(&repay_reserve_info.data.borrow())?;
    if repay_reserve_info.owner != program_id {
        msg!("Repay reserve provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &repay_reserve.lending_market != lending_market_info.key {
        msg!("Repay reserve lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &repay_reserve.liquidity.supply_pubkey == source_liquidity_info.key {
        msg!("Repay reserve liquidity supply cannot be used as the source liquidity provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &repay_reserve.liquidity.supply_pubkey != destination_liquidity_info.key {
        msg!("Repay reserve liquidity supply must be used as the destination liquidity provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if repay_reserve.last_update.is_stale(clock.slot)? {
        msg!("Repay reserve is stale and must be refreshed in the current slot");
        return Err(LendingError::ReserveStale.into());
    }

    let mut obligation = Obligation::unpack(&obligation_info.data.borrow())?;
    if obligation_info.owner != program_id {
        msg!("Obligation provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &obligation.lending_market != lending_market_info.key {
        msg!("Obligation lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }

    let (liquidity, liquidity_index) =
        obligation.find_liquidity_in_borrows_mut(*repay_reserve_info.key)?;
    if liquidity.borrowed_amount_wads == Decimal::zero() {
        msg!("Liquidity borrowed amount is zero");
        return Err(LendingError::ObligationLiquidityEmpty.into());
    }

    // refreshing specific borrow instead of checking obligation stale
    liquidity.accrue_interest(repay_reserve.liquidity.cumulative_borrow_rate_wads)?;

    let CalculateRepayResult {
        settle_amount,
        repay_amount,
    } = repay_reserve.calculate_repay(liquidity_amount, liquidity.borrowed_amount_wads)?;

    if repay_amount == 0 {
        msg!("Repay amount is too small to transfer liquidity");
        return Err(LendingError::RepayTooSmall.into());
    }

    repay_reserve.liquidity.repay(repay_amount, settle_amount)?;
    repay_reserve.last_update.mark_stale();
    Reserve::pack(repay_reserve, &mut repay_reserve_info.data.borrow_mut())?;

    obligation.repay(settle_amount, liquidity_index)?;
    obligation.last_update.mark_stale();
    Obligation::pack(obligation, &mut obligation_info.data.borrow_mut())?;

    spl_token_transfer(TokenTransferParams {
        source: source_liquidity_info.clone(),
        destination: destination_liquidity_info.clone(),
        amount: repay_amount,
        authority: user_transfer_authority_info.clone(),
        authority_signer_seeds: &[],
        token_program: token_program_id.clone(),
    })?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _liquidate_obligation<'a>(
    program_id: &Pubkey,
    liquidity_amount: u64,
    source_liquidity_info: &AccountInfo<'a>,
    destination_collateral_info: &AccountInfo<'a>,
    repay_reserve_info: &AccountInfo<'a>,
    repay_reserve_liquidity_supply_info: &AccountInfo<'a>,
    withdraw_reserve_info: &AccountInfo<'a>,
    withdraw_reserve_collateral_supply_info: &AccountInfo<'a>,
    obligation_info: &AccountInfo<'a>,
    lending_market_info: &AccountInfo<'a>,
    lending_market_authority_info: &AccountInfo<'a>,
    user_transfer_authority_info: &AccountInfo<'a>,
    clock: &Clock,
    token_program_id: &AccountInfo<'a>,
) -> Result<u64, ProgramError> {
    let lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        msg!("Lending market provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.token_program_id != token_program_id.key {
        msg!("Lending market token program does not match the token program provided");
        return Err(LendingError::InvalidTokenProgram.into());
    }

    let mut repay_reserve = Reserve::unpack(&repay_reserve_info.data.borrow())?;
    if repay_reserve_info.owner != program_id {
        msg!("Repay reserve provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &repay_reserve.lending_market != lending_market_info.key {
        msg!("Repay reserve lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &repay_reserve.liquidity.supply_pubkey != repay_reserve_liquidity_supply_info.key {
        msg!("Repay reserve liquidity supply does not match the repay reserve liquidity supply provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &repay_reserve.liquidity.supply_pubkey == source_liquidity_info.key {
        msg!("Repay reserve liquidity supply cannot be used as the source liquidity provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &repay_reserve.collateral.supply_pubkey == destination_collateral_info.key {
        msg!(
            "Repay reserve collateral supply cannot be used as the destination collateral provided"
        );
        return Err(LendingError::InvalidAccountInput.into());
    }
    if repay_reserve.last_update.is_stale(clock.slot)? {
        msg!("Repay reserve is stale and must be refreshed in the current slot");
        return Err(LendingError::ReserveStale.into());
    }

    let withdraw_reserve = Reserve::unpack(&withdraw_reserve_info.data.borrow())?;
    if withdraw_reserve_info.owner != program_id {
        msg!("Withdraw reserve provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &withdraw_reserve.lending_market != lending_market_info.key {
        msg!("Withdraw reserve lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &withdraw_reserve.collateral.supply_pubkey != withdraw_reserve_collateral_supply_info.key {
        msg!("Withdraw reserve collateral supply does not match the withdraw reserve collateral supply provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &withdraw_reserve.liquidity.supply_pubkey == source_liquidity_info.key {
        msg!("Withdraw reserve liquidity supply cannot be used as the source liquidity provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &withdraw_reserve.collateral.supply_pubkey == destination_collateral_info.key {
        msg!("Withdraw reserve collateral supply cannot be used as the destination collateral provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if withdraw_reserve.last_update.is_stale(clock.slot)? {
        msg!("Withdraw reserve is stale and must be refreshed in the current slot");
        return Err(LendingError::ReserveStale.into());
    }

    let mut obligation = Obligation::unpack(&obligation_info.data.borrow())?;
    if obligation_info.owner != program_id {
        msg!("Obligation provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &obligation.lending_market != lending_market_info.key {
        msg!("Obligation lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if obligation.last_update.is_stale(clock.slot)? {
        msg!("Obligation is stale and must be refreshed in the current slot");
        return Err(LendingError::ObligationStale.into());
    }
    if obligation.deposited_value == Decimal::zero() {
        msg!("Obligation deposited value is zero");
        return Err(LendingError::ObligationDepositsZero.into());
    }
    if obligation.borrowed_value == Decimal::zero() {
        msg!("Obligation borrowed value is zero");
        return Err(LendingError::ObligationBorrowsZero.into());
    }
    if obligation.borrowed_value < obligation.unhealthy_borrow_value {
        msg!("Obligation is healthy and cannot be liquidated");
        return Err(LendingError::ObligationHealthy.into());
    }

    let (liquidity, liquidity_index) =
        obligation.find_liquidity_in_borrows(*repay_reserve_info.key)?;
    if liquidity.market_value == Decimal::zero() {
        msg!("Obligation borrow value is zero");
        return Err(LendingError::ObligationLiquidityEmpty.into());
    }

    let (collateral, collateral_index) =
        obligation.find_collateral_in_deposits(*withdraw_reserve_info.key)?;
    if collateral.market_value == Decimal::zero() {
        msg!("Obligation deposit value is zero");
        return Err(LendingError::ObligationCollateralEmpty.into());
    }

    let authority_signer_seeds = &[
        lending_market_info.key.as_ref(),
        &[lending_market.bump_seed],
    ];
    let lending_market_authority_pubkey =
        Pubkey::create_program_address(authority_signer_seeds, program_id)?;
    if &lending_market_authority_pubkey != lending_market_authority_info.key {
        msg!(
            "Derived lending market authority does not match the lending market authority provided"
        );
        return Err(LendingError::InvalidMarketAuthority.into());
    }

    let CalculateLiquidationResult {
        settle_amount,
        repay_amount,
        withdraw_amount,
    } = withdraw_reserve.calculate_liquidation(
        liquidity_amount,
        &obligation,
        liquidity,
        collateral,
    )?;

    if repay_amount == 0 {
        msg!("Liquidation is too small to transfer liquidity");
        return Err(LendingError::LiquidationTooSmall.into());
    }
    if withdraw_amount == 0 {
        msg!("Liquidation is too small to receive collateral");
        return Err(LendingError::LiquidationTooSmall.into());
    }

    repay_reserve.liquidity.repay(repay_amount, settle_amount)?;
    repay_reserve.last_update.mark_stale();
    Reserve::pack(repay_reserve, &mut repay_reserve_info.data.borrow_mut())?;

    obligation.repay(settle_amount, liquidity_index)?;
    obligation.withdraw(withdraw_amount, collateral_index)?;
    obligation.last_update.mark_stale();
    Obligation::pack(obligation, &mut obligation_info.data.borrow_mut())?;

    spl_token_transfer(TokenTransferParams {
        source: source_liquidity_info.clone(),
        destination: repay_reserve_liquidity_supply_info.clone(),
        amount: repay_amount,
        authority: user_transfer_authority_info.clone(),
        authority_signer_seeds: &[],
        token_program: token_program_id.clone(),
    })?;

    spl_token_transfer(TokenTransferParams {
        source: withdraw_reserve_collateral_supply_info.clone(),
        destination: destination_collateral_info.clone(),
        amount: withdraw_amount,
        authority: lending_market_authority_info.clone(),
        authority_signer_seeds,
        token_program: token_program_id.clone(),
    })?;

    Ok(withdraw_amount)
}

#[inline(never)] // avoid stack frame limit
fn process_liquidate_obligation_and_redeem_reserve_collateral(
    program_id: &Pubkey,
    liquidity_amount: u64,
    accounts: &[AccountInfo],
) -> ProgramResult {
    if liquidity_amount == 0 {
        msg!("Liquidity amount provided cannot be zero");
        return Err(LendingError::InvalidAmount.into());
    }

    let account_info_iter = &mut accounts.iter();
    let source_liquidity_info = next_account_info(account_info_iter)?;
    let destination_collateral_info = next_account_info(account_info_iter)?;
    let destination_liquidity_info = next_account_info(account_info_iter)?;
    let repay_reserve_info = next_account_info(account_info_iter)?;
    let repay_reserve_liquidity_supply_info = next_account_info(account_info_iter)?;
    let withdraw_reserve_info = next_account_info(account_info_iter)?;
    let withdraw_reserve_collateral_mint_info = next_account_info(account_info_iter)?;
    let withdraw_reserve_collateral_supply_info = next_account_info(account_info_iter)?;
    let withdraw_reserve_liquidity_supply_info = next_account_info(account_info_iter)?;
    let withdraw_reserve_liquidity_fee_receiver_info = next_account_info(account_info_iter)?;
    let obligation_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let lending_market_authority_info = next_account_info(account_info_iter)?;
    let user_transfer_authority_info = next_account_info(account_info_iter)?;
    let token_program_id = next_account_info(account_info_iter)?;
    let clock = &Clock::get()?;

    let withdrawn_collateral_amount = _liquidate_obligation(
        program_id,
        liquidity_amount,
        source_liquidity_info,
        destination_collateral_info,
        repay_reserve_info,
        repay_reserve_liquidity_supply_info,
        withdraw_reserve_info,
        withdraw_reserve_collateral_supply_info,
        obligation_info,
        lending_market_info,
        lending_market_authority_info,
        user_transfer_authority_info,
        clock,
        token_program_id,
    )?;

    _refresh_reserve_interest(program_id, withdraw_reserve_info, clock)?;
    let withdraw_reserve = Reserve::unpack(&withdraw_reserve_info.data.borrow())?;
    let collateral_exchange_rate = withdraw_reserve.collateral_exchange_rate()?;
    let max_redeemable_collateral = collateral_exchange_rate
        .liquidity_to_collateral(withdraw_reserve.liquidity.available_amount)?;
    let withdraw_collateral_amount = min(withdrawn_collateral_amount, max_redeemable_collateral);
    // if there is liquidity redeem it
    if withdraw_collateral_amount != 0 {
        let withdraw_liquidity_amount = _redeem_reserve_collateral(
            program_id,
            withdraw_collateral_amount,
            destination_collateral_info,
            destination_liquidity_info,
            withdraw_reserve_info,
            withdraw_reserve_collateral_mint_info,
            withdraw_reserve_liquidity_supply_info,
            lending_market_info,
            lending_market_authority_info,
            user_transfer_authority_info,
            clock,
            token_program_id,
        )?;
        let withdraw_reserve = Reserve::unpack(&withdraw_reserve_info.data.borrow())?;
        if &withdraw_reserve.config.fee_receiver != withdraw_reserve_liquidity_fee_receiver_info.key
        {
            msg!("Withdraw reserve liquidity fee receiver does not match the reserve liquidity fee receiver provided");
            return Err(LendingError::InvalidAccountInput.into());
        }
        let protocol_fee =
            withdraw_reserve.calculate_protocol_liquidation_fee(withdraw_liquidity_amount)?;

        spl_token_transfer(TokenTransferParams {
            source: destination_liquidity_info.clone(),
            destination: withdraw_reserve_liquidity_fee_receiver_info.clone(),
            amount: protocol_fee,
            authority: user_transfer_authority_info.clone(),
            authority_signer_seeds: &[],
            token_program: token_program_id.clone(),
        })?;
    }

    Ok(())
}

#[inline(never)] // avoid stack frame limit
fn process_withdraw_obligation_collateral_and_redeem_reserve_liquidity(
    program_id: &Pubkey,
    collateral_amount: u64,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().peekable();
    let reserve_collateral_info = next_account_info(account_info_iter)?;
    let user_collateral_info = next_account_info(account_info_iter)?;
    let reserve_info = next_account_info(account_info_iter)?;
    let obligation_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let lending_market_authority_info = next_account_info(account_info_iter)?;
    let user_liquidity_info = next_account_info(account_info_iter)?;
    let reserve_collateral_mint_info = next_account_info(account_info_iter)?;
    let reserve_liquidity_supply_info = next_account_info(account_info_iter)?;
    let obligation_owner_info = next_account_info(account_info_iter)?;
    let user_transfer_authority_info = next_account_info(account_info_iter)?;
    let clock = &Clock::get()?;
    if account_info_iter.peek().map(|a| a.key) == Some(&clock::ID) {
        next_account_info(account_info_iter)?;
    }
    let token_program_id = next_account_info(account_info_iter)?;

    let liquidity_amount = _withdraw_obligation_collateral(
        program_id,
        collateral_amount,
        reserve_collateral_info,
        user_collateral_info,
        reserve_info,
        obligation_info,
        lending_market_info,
        lending_market_authority_info,
        obligation_owner_info,
        clock,
        token_program_id,
    )?;

    _redeem_reserve_collateral(
        program_id,
        liquidity_amount,
        user_collateral_info,
        user_liquidity_info,
        reserve_info,
        reserve_collateral_mint_info,
        reserve_liquidity_supply_info,
        lending_market_info,
        lending_market_authority_info,
        user_transfer_authority_info,
        clock,
        token_program_id,
    )?;
    Ok(())
}

#[inline(never)] // avoid stack frame limit
fn process_update_reserve_config(
    program_id: &Pubkey,
    config: ReserveConfig,
    accounts: &[AccountInfo],
) -> ProgramResult {
    validate_reserve_config(config)?;
    let account_info_iter = &mut accounts.iter();
    let reserve_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let lending_market_authority_info = next_account_info(account_info_iter)?;
    let lending_market_owner_info = next_account_info(account_info_iter)?;
    let pyth_product_info = next_account_info(account_info_iter)?;
    let pyth_price_info = next_account_info(account_info_iter)?;
    let switchboard_feed_info = next_account_info(account_info_iter)?;

    let mut reserve = Reserve::unpack(&reserve_info.data.borrow())?;
    if reserve_info.owner != program_id {
        msg!(
            "Reserve provided is not owned by the lending program {} != {}",
            &reserve_info.owner.to_string(),
            &program_id.to_string(),
        );
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &reserve.lending_market != lending_market_info.key {
        msg!("Reserve lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }

    let lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        msg!(
            "Lending market provided is not owned by the lending program  {} != {}",
            &lending_market_info.owner.to_string(),
            &program_id.to_string(),
        );
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.owner != lending_market_owner_info.key {
        msg!("Lending market owner does not match the lending market owner provided");
        return Err(LendingError::InvalidMarketOwner.into());
    }
    if !lending_market_owner_info.is_signer {
        msg!("Lending market owner provided must be a signer");
        return Err(LendingError::InvalidSigner.into());
    }

    let authority_signer_seeds = &[
        lending_market_info.key.as_ref(),
        &[lending_market.bump_seed],
    ];
    let lending_market_authority_pubkey =
        Pubkey::create_program_address(authority_signer_seeds, program_id)?;
    if &lending_market_authority_pubkey != lending_market_authority_info.key {
        msg!(
            "Derived lending market authority does not match the lending market authority provided"
        );
        return Err(LendingError::InvalidMarketAuthority.into());
    }

    if *pyth_price_info.key != reserve.liquidity.pyth_oracle_pubkey {
        validate_pyth_keys(&lending_market, pyth_product_info, pyth_price_info)?;
        reserve.liquidity.pyth_oracle_pubkey = *pyth_price_info.key;
    }

    if *switchboard_feed_info.key != reserve.liquidity.switchboard_oracle_pubkey {
        validate_switchboard_keys(&lending_market, switchboard_feed_info)?;
        reserve.liquidity.switchboard_oracle_pubkey = *switchboard_feed_info.key;
    }
    if reserve.liquidity.switchboard_oracle_pubkey == solend_program::NULL_PUBKEY
        && (*pyth_price_info.key == solend_program::NULL_PUBKEY
            || *pyth_product_info.key == solend_program::NULL_PUBKEY)
    {
        msg!("At least one price oracle must have a non-null pubkey");
        return Err(LendingError::InvalidOracleConfig.into());
    }

    reserve.config = config;
    Reserve::pack(reserve, &mut reserve_info.data.borrow_mut())?;
    Ok(())
}

#[inline(never)] // avoid stack frame limit
fn process_redeem_fees(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().peekable();
    let reserve_info = next_account_info(account_info_iter)?;
    let reserve_liquidity_fee_receiver_info = next_account_info(account_info_iter)?;
    let reserve_supply_liquidity_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let lending_market_authority_info = next_account_info(account_info_iter)?;
    let token_program_id = next_account_info(account_info_iter)?;
    let clock = &Clock::get()?;

    let mut reserve = Reserve::unpack(&reserve_info.data.borrow())?;
    if reserve_info.owner != program_id {
        msg!(
            "Reserve provided is not owned by the lending program {} != {}",
            &reserve_info.owner.to_string(),
            &program_id.to_string(),
        );
        return Err(LendingError::InvalidAccountOwner.into());
    }

    if &reserve.config.fee_receiver != reserve_liquidity_fee_receiver_info.key {
        msg!("Reserve liquidity fee receiver does not match the reserve liquidity fee receiver provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &reserve.liquidity.supply_pubkey != reserve_supply_liquidity_info.key {
        msg!("Reserve liquidity supply must be used as the reserve supply liquidity provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &reserve.lending_market != lending_market_info.key {
        msg!("Reserve lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if reserve.last_update.is_stale(clock.slot)? {
        msg!("reserve is stale and must be refreshed in the current slot");
        return Err(LendingError::ReserveStale.into());
    }

    let lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        msg!("Lending market provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.token_program_id != token_program_id.key {
        msg!("Lending market token program does not match the token program provided");
        return Err(LendingError::InvalidTokenProgram.into());
    }
    let authority_signer_seeds = &[
        lending_market_info.key.as_ref(),
        &[lending_market.bump_seed],
    ];
    let lending_market_authority_pubkey =
        Pubkey::create_program_address(authority_signer_seeds, program_id)?;
    if &lending_market_authority_pubkey != lending_market_authority_info.key {
        msg!(
            "Derived lending market authority does not match the lending market authority provided"
        );
        return Err(LendingError::InvalidMarketAuthority.into());
    }

    let withdraw_amount = reserve.calculate_redeem_fees()?;
    if withdraw_amount == 0 {
        return Err(LendingError::InsufficientProtocolFeesToRedeem.into());
    }

    reserve.liquidity.redeem_fees(withdraw_amount)?;
    reserve.last_update.mark_stale();
    Reserve::pack(reserve, &mut reserve_info.data.borrow_mut())?;

    spl_token_transfer(TokenTransferParams {
        source: reserve_supply_liquidity_info.clone(),
        destination: reserve_liquidity_fee_receiver_info.clone(),
        amount: withdraw_amount,
        authority: lending_market_authority_info.clone(),
        authority_signer_seeds,
        token_program: token_program_id.clone(),
    })?;

    Ok(())
}

fn process_flash_borrow_reserve_liquidity(
    program_id: &Pubkey,
    liquidity_amount: u64,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let source_liquidity_info = next_account_info(account_info_iter)?;
    let destination_liquidity_info = next_account_info(account_info_iter)?;
    let reserve_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let lending_market_authority_info = next_account_info(account_info_iter)?;
    let sysvar_info = next_account_info(account_info_iter)?;
    let token_program_id = next_account_info(account_info_iter)?;
    let clock = Clock::get()?;

    _refresh_reserve_interest(program_id, reserve_info, &clock)?;
    _flash_borrow_reserve_liquidity(
        program_id,
        liquidity_amount,
        source_liquidity_info,
        destination_liquidity_info,
        reserve_info,
        lending_market_info,
        lending_market_authority_info,
        sysvar_info,
        token_program_id,
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _flash_borrow_reserve_liquidity<'a>(
    program_id: &Pubkey,
    liquidity_amount: u64,
    source_liquidity_info: &AccountInfo<'a>,
    destination_liquidity_info: &AccountInfo<'a>,
    reserve_info: &AccountInfo<'a>,
    lending_market_info: &AccountInfo<'a>,
    lending_market_authority_info: &AccountInfo<'a>,
    sysvar_info: &AccountInfo<'a>,
    token_program_id: &AccountInfo<'a>,
) -> ProgramResult {
    let lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        msg!("Lending market provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.token_program_id != token_program_id.key {
        msg!("Lending market token program does not match the token program provided");
        return Err(LendingError::InvalidTokenProgram.into());
    }
    let mut reserve = Reserve::unpack(&reserve_info.data.borrow())?;
    if reserve_info.owner != program_id {
        msg!("Reserve provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &reserve.lending_market != lending_market_info.key {
        msg!("Reserve lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &reserve.liquidity.supply_pubkey != source_liquidity_info.key {
        msg!("Borrow reserve liquidity supply must be used as the source liquidity provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &reserve.liquidity.supply_pubkey == destination_liquidity_info.key {
        msg!(
            "Borrow reserve liquidity supply cannot be used as the destination liquidity provided"
        );
        return Err(LendingError::InvalidAccountInput.into());
    }
    let authority_signer_seeds = &[
        lending_market_info.key.as_ref(),
        &[lending_market.bump_seed],
    ];
    let lending_market_authority_pubkey =
        Pubkey::create_program_address(authority_signer_seeds, program_id)?;
    if &lending_market_authority_pubkey != lending_market_authority_info.key {
        msg!(
            "Derived lending market authority {} does not match the lending market authority provided {}",
            &lending_market_authority_pubkey.to_string(),
            &lending_market_authority_info.key.to_string(),
        );
        return Err(LendingError::InvalidMarketAuthority.into());
    }

    if reserve.config.fees.flash_loan_fee_wad == u64::MAX {
        msg!("Flash loans are disabled for this reserve");
        return Err(LendingError::FlashLoansDisabled.into());
    }

    if Decimal::from(liquidity_amount)
        .try_add(reserve.liquidity.borrowed_amount_wads)?
        .try_floor_u64()?
        > reserve.config.borrow_limit
    {
        msg!("Cannot borrow above the borrow limit");
        return Err(LendingError::InvalidAmount.into());
    }

    // Make sure this isnt a cpi call
    let current_index = load_current_index_checked(sysvar_info)? as usize;
    if is_cpi_call(program_id, current_index, sysvar_info)? {
        msg!("Flash Borrow was called via CPI!");
        return Err(LendingError::FlashBorrowCpi.into());
    }

    // Find and validate the flash repay instruction.
    //
    // 1. Ensure the instruction is for this program
    // 2. Ensure the instruction can be unpacked into a LendingInstruction
    // 3. Ensure that the reserve for the repay matches the borrow
    // 4. Ensure that there are no other flash instructions in the rest of the transaction
    // 5. Ensure that the repay amount matches the borrow amount
    //
    // If all of these conditions are not met, the flash borrow fails.
    let mut i = current_index;
    let mut found_repay_ix = false;

    loop {
        i += 1;

        let ixn = match load_instruction_at_checked(i, sysvar_info) {
            Ok(ix) => ix,
            Err(ProgramError::InvalidArgument) => break, // out of bounds
            Err(e) => {
                return Err(e);
            }
        };

        if ixn.program_id != *program_id {
            continue;
        }

        let unpacked = LendingInstruction::unpack(ixn.data.as_slice())?;
        match unpacked {
            LendingInstruction::FlashRepayReserveLiquidity {
                liquidity_amount: repay_liquidity_amount,
                borrow_instruction_index,
            } => {
                if found_repay_ix {
                    msg!("Multiple flash repays not allowed");
                    return Err(LendingError::MultipleFlashBorrows.into());
                }
                if ixn.accounts[4].pubkey != *reserve_info.key {
                    msg!("Invalid reserve account on flash repay");
                    return Err(LendingError::InvalidFlashRepay.into());
                }
                if repay_liquidity_amount != liquidity_amount {
                    msg!("Liquidity amount for flash repay doesn't match borrow");
                    return Err(LendingError::InvalidFlashRepay.into());
                }
                if (borrow_instruction_index as usize) != current_index {
                    msg!("Borrow instruction index {} for flash repay doesn't match current index {}", borrow_instruction_index, current_index);
                    return Err(LendingError::InvalidFlashRepay.into());
                }

                found_repay_ix = true;
            }
            LendingInstruction::FlashBorrowReserveLiquidity { .. } => {
                msg!("Multiple flash borrows not allowed");
                return Err(LendingError::MultipleFlashBorrows.into());
            }
            _ => (),
        };
    }

    if !found_repay_ix {
        msg!("No flash repay found");
        return Err(LendingError::NoFlashRepayFound.into());
    }

    reserve.liquidity.borrow(Decimal::from(liquidity_amount))?;
    reserve.last_update.mark_stale();
    Reserve::pack(reserve, &mut reserve_info.data.borrow_mut())?;

    spl_token_transfer(TokenTransferParams {
        source: source_liquidity_info.clone(),
        destination: destination_liquidity_info.clone(),
        amount: liquidity_amount,
        authority: lending_market_authority_info.clone(),
        authority_signer_seeds,
        token_program: token_program_id.clone(),
    })?;

    Ok(())
}

fn process_flash_repay_reserve_liquidity(
    program_id: &Pubkey,
    liquidity_amount: u64,
    borrow_instruction_index: u8,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let source_liquidity_info = next_account_info(account_info_iter)?;
    let destination_liquidity_info = next_account_info(account_info_iter)?;
    let reserve_liquidity_fee_receiver_info = next_account_info(account_info_iter)?;
    let host_fee_receiver_info = next_account_info(account_info_iter)?;
    let reserve_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let user_transfer_authority_info = next_account_info(account_info_iter)?;
    let sysvar_info = next_account_info(account_info_iter)?;
    let token_program_id = next_account_info(account_info_iter)?;

    _flash_repay_reserve_liquidity(
        program_id,
        liquidity_amount,
        borrow_instruction_index,
        source_liquidity_info,
        destination_liquidity_info,
        reserve_liquidity_fee_receiver_info,
        host_fee_receiver_info,
        reserve_info,
        lending_market_info,
        user_transfer_authority_info,
        sysvar_info,
        token_program_id,
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _flash_repay_reserve_liquidity<'a>(
    program_id: &Pubkey,
    liquidity_amount: u64,
    borrow_instruction_index: u8,
    source_liquidity_info: &AccountInfo<'a>,
    destination_liquidity_info: &AccountInfo<'a>,
    reserve_liquidity_fee_receiver_info: &AccountInfo<'a>,
    host_fee_receiver_info: &AccountInfo<'a>,
    reserve_info: &AccountInfo<'a>,
    lending_market_info: &AccountInfo<'a>,
    user_transfer_authority_info: &AccountInfo<'a>,
    sysvar_info: &AccountInfo<'a>,
    token_program_id: &AccountInfo<'a>,
) -> ProgramResult {
    let lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        msg!("Lending market provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.token_program_id != token_program_id.key {
        msg!("Lending market token program does not match the token program provided");
        return Err(LendingError::InvalidTokenProgram.into());
    }
    let mut reserve = Reserve::unpack(&reserve_info.data.borrow())?;
    if reserve_info.owner != program_id {
        msg!("Reserve provided is not owned by the lending program");
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &reserve.lending_market != lending_market_info.key {
        msg!("Reserve lending market does not match the lending market provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &reserve.liquidity.supply_pubkey != destination_liquidity_info.key {
        msg!("Reserve liquidity supply does not match the reserve liquidity supply provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &reserve.liquidity.supply_pubkey == source_liquidity_info.key {
        msg!("Reserve liquidity supply cannot be used as the source liquidity provided");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &reserve.config.fee_receiver != reserve_liquidity_fee_receiver_info.key {
        msg!("Reserve liquidity fee receiver does not match the reserve liquidity fee receiver provided");
        return Err(LendingError::InvalidAccountInput.into());
    }

    let flash_loan_amount = liquidity_amount;

    let flash_loan_amount_decimal = Decimal::from(flash_loan_amount);
    let (origination_fee, host_fee) = reserve
        .config
        .fees
        .calculate_flash_loan_fees(flash_loan_amount_decimal)?;

    // Make sure this isnt a cpi call
    let current_index = load_current_index_checked(sysvar_info)? as usize;
    if is_cpi_call(program_id, current_index, sysvar_info)? {
        msg!("Flash Repay was called via CPI!");
        return Err(LendingError::FlashRepayCpi.into());
    }

    // validate flash borrow
    if (borrow_instruction_index as usize) > current_index {
        msg!(
            "Flash repay: borrow instruction index {} has to be less than current index {}",
            borrow_instruction_index,
            current_index
        );
        return Err(LendingError::InvalidFlashRepay.into());
    }

    let ixn = load_instruction_at_checked(borrow_instruction_index as usize, sysvar_info)?;
    if ixn.program_id != *program_id {
        msg!(
            "Flash repay: supplied instruction index {} doesn't belong to program id {}",
            borrow_instruction_index,
            *program_id
        );
        return Err(LendingError::InvalidFlashRepay.into());
    }

    let unpacked = LendingInstruction::unpack(ixn.data.as_slice())?;
    match unpacked {
        LendingInstruction::FlashBorrowReserveLiquidity {
            liquidity_amount: borrow_liquidity_amount,
        } => {
            // re-check everything here out of paranoia
            if ixn.accounts[2].pubkey != *reserve_info.key {
                msg!("Invalid reserve account on flash repay");
                return Err(LendingError::InvalidFlashRepay.into());
            }

            if liquidity_amount != borrow_liquidity_amount {
                msg!("Liquidity amount for flash repay doesn't match borrow");
                return Err(LendingError::InvalidFlashRepay.into());
            }
        }
        _ => {
            msg!("Flash repay: Supplied borrow instruction index is not a flash borrow");
            return Err(LendingError::InvalidFlashRepay.into());
        }
    };

    reserve
        .liquidity
        .repay(flash_loan_amount, flash_loan_amount_decimal)?;
    reserve.last_update.mark_stale();
    Reserve::pack(reserve, &mut reserve_info.data.borrow_mut())?;

    spl_token_transfer(TokenTransferParams {
        source: source_liquidity_info.clone(),
        destination: destination_liquidity_info.clone(),
        amount: flash_loan_amount,
        authority: user_transfer_authority_info.clone(),
        authority_signer_seeds: &[],
        token_program: token_program_id.clone(),
    })?;

    if host_fee > 0 {
        spl_token_transfer(TokenTransferParams {
            source: source_liquidity_info.clone(),
            destination: host_fee_receiver_info.clone(),
            amount: host_fee,
            authority: user_transfer_authority_info.clone(),
            authority_signer_seeds: &[],
            token_program: token_program_id.clone(),
        })?;
    }

    if origination_fee > 0 {
        spl_token_transfer(TokenTransferParams {
            source: source_liquidity_info.clone(),
            destination: reserve_liquidity_fee_receiver_info.clone(),
            amount: origination_fee,
            authority: user_transfer_authority_info.clone(),
            authority_signer_seeds: &[],
            token_program: token_program_id.clone(),
        })?;
    }

    Ok(())
}

fn assert_rent_exempt(rent: &Rent, account_info: &AccountInfo) -> ProgramResult {
    if !rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        msg!(
            "Rent exempt balance insufficient got {} expected {}",
            &account_info.lamports().to_string(),
            &rent.minimum_balance(account_info.data_len()).to_string(),
        );
        Err(LendingError::NotRentExempt.into())
    } else {
        Ok(())
    }
}

fn assert_uninitialized<T: Pack + IsInitialized>(
    account_info: &AccountInfo,
) -> Result<T, ProgramError> {
    let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
    if account.is_initialized() {
        Err(LendingError::AlreadyInitialized.into())
    } else {
        Ok(account)
    }
}

/// Unpacks a spl_token `Mint`.
fn unpack_mint(data: &[u8]) -> Result<Mint, LendingError> {
    Mint::unpack(data).map_err(|_| LendingError::InvalidTokenMint)
}

fn get_pyth_product_quote_currency(
    pyth_product: &ProductAccount,
) -> Result<[u8; 32], ProgramError> {
    pyth_product
        .iter()
        .find_map(|(key, val)| {
            if key == "quote_currency" {
                let mut value = [0u8; 32];
                value[0..val.len()].copy_from_slice(val.as_bytes());
                Some(value)
            } else {
                None
            }
        })
        .ok_or_else(|| {
            msg!("Pyth product quote currency not found");
            LendingError::InvalidOracleConfig.into()
        })
}

fn get_price(
    switchboard_feed_info: Option<&AccountInfo>,
    pyth_price_account_info: &AccountInfo,
    clock: &Clock,
) -> Result<Decimal, ProgramError> {
    let pyth_price = get_pyth_price(pyth_price_account_info, clock).unwrap_or_default();
    if pyth_price != Decimal::zero() {
        return Ok(pyth_price);
    }

    // if switchboard was not passed in don't try to grab the price
    if let Some(switchboard_feed_info_unwrapped) = switchboard_feed_info {
        return get_switchboard_price(switchboard_feed_info_unwrapped, clock);
    }

    Err(LendingError::InvalidOracleConfig.into())
}

fn get_switchboard_price(
    switchboard_feed_info: &AccountInfo,
    clock: &Clock,
) -> Result<Decimal, ProgramError> {
    const STALE_AFTER_SLOTS_ELAPSED: u64 = 240;

    if *switchboard_feed_info.key == solend_program::NULL_PUBKEY {
        return Err(LendingError::NullOracleConfig.into());
    }
    if switchboard_feed_info.owner == &switchboard_v2_mainnet::id()
        || switchboard_feed_info.owner == &switchboard_v2_devnet::id()
    {
        return get_switchboard_price_v2(switchboard_feed_info, clock);
    }

    let account_buf = switchboard_feed_info.try_borrow_data()?;
    // first byte type discriminator
    if account_buf[0] != SwitchboardAccountType::TYPE_AGGREGATOR as u8 {
        msg!("switchboard address not of type aggregator");
        return Err(LendingError::InvalidAccountInput.into());
    }

    let aggregator: AggregatorState = get_aggregator(switchboard_feed_info)?;
    // if aggregator.version != 1 {
    //     msg!("switchboard version incorrect");
    //     return Err(LendingError::InvalidAccountInput.into());
    // }
    let round_result: RoundResult = get_aggregator_result(&aggregator)?;

    let slots_elapsed = clock
        .slot
        .checked_sub(round_result.round_open_slot.unwrap())
        .ok_or(LendingError::MathOverflow)?;
    if slots_elapsed >= STALE_AFTER_SLOTS_ELAPSED {
        msg!("Switchboard oracle price is stale");
        return Err(LendingError::InvalidOracleConfig.into());
    }

    let price_float = round_result.result.unwrap_or(0.0);

    // we just do this so we can parse coins with low usd value
    // it might be better to just extract the mantissa and exponent from the float directly
    let price_quotient = 10u64.pow(9);
    let price = ((price_quotient as f64) * price_float) as u128;

    Decimal::from(price).try_div(price_quotient)
}

fn get_switchboard_price_v2(
    switchboard_feed_info: &AccountInfo,
    clock: &Clock,
) -> Result<Decimal, ProgramError> {
    const STALE_AFTER_SLOTS_ELAPSED: u64 = 240;
    let data = &switchboard_feed_info.try_borrow_data()?;
    let feed = AggregatorAccountData::new_from_bytes(data)?;

    let slots_elapsed = clock
        .slot
        .checked_sub(feed.latest_confirmed_round.round_open_slot)
        .ok_or(LendingError::MathOverflow)?;
    if slots_elapsed >= STALE_AFTER_SLOTS_ELAPSED {
        msg!("Switchboard oracle price is stale");
        return Err(LendingError::InvalidOracleConfig.into());
    }

    let price_switchboard_desc = feed.get_result()?;
    if price_switchboard_desc.mantissa < 0 {
        msg!("Switchboard oracle price is negative which is not allowed");
        return Err(LendingError::InvalidOracleConfig.into());
    }
    let price = Decimal::from(price_switchboard_desc.mantissa as u128);
    let exp = Decimal::from((10u128).checked_pow(price_switchboard_desc.scale).unwrap());
    price.try_div(exp)
}

/// Issue a spl_token `InitializeAccount` instruction.
#[inline(always)]
fn spl_token_init_account(params: TokenInitializeAccountParams<'_>) -> ProgramResult {
    let TokenInitializeAccountParams {
        account,
        mint,
        owner,
        rent,
        token_program,
    } = params;
    let ix = spl_token::instruction::initialize_account(
        token_program.key,
        account.key,
        mint.key,
        owner.key,
    )?;
    let result = invoke(&ix, &[account, mint, owner, rent, token_program]);
    result.map_err(|_| LendingError::TokenInitializeAccountFailed.into())
}

/// Issue a spl_token `InitializeMint` instruction.
#[inline(always)]
fn spl_token_init_mint(params: TokenInitializeMintParams<'_, '_>) -> ProgramResult {
    let TokenInitializeMintParams {
        mint,
        rent,
        authority,
        token_program,
        decimals,
    } = params;
    let ix = spl_token::instruction::initialize_mint(
        token_program.key,
        mint.key,
        authority,
        None,
        decimals,
    )?;
    let result = invoke(&ix, &[mint, rent, token_program]);
    result.map_err(|_| LendingError::TokenInitializeMintFailed.into())
}

/// Invoke signed unless signers seeds are empty
#[inline(always)]
fn invoke_optionally_signed(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
    authority_signer_seeds: &[&[u8]],
) -> ProgramResult {
    if authority_signer_seeds.is_empty() {
        invoke(instruction, account_infos)
    } else {
        invoke_signed(instruction, account_infos, &[authority_signer_seeds])
    }
}

/// Issue a spl_token `Transfer` instruction.
#[inline(always)]
fn spl_token_transfer(params: TokenTransferParams<'_, '_>) -> ProgramResult {
    let TokenTransferParams {
        source,
        destination,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;
    let result = invoke_optionally_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, destination, authority, token_program],
        authority_signer_seeds,
    );

    result.map_err(|_| LendingError::TokenTransferFailed.into())
}

/// Issue a spl_token `MintTo` instruction.
fn spl_token_mint_to(params: TokenMintToParams<'_, '_>) -> ProgramResult {
    let TokenMintToParams {
        mint,
        destination,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;
    let result = invoke_optionally_signed(
        &spl_token::instruction::mint_to(
            token_program.key,
            mint.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[mint, destination, authority, token_program],
        authority_signer_seeds,
    );
    result.map_err(|_| LendingError::TokenMintToFailed.into())
}

/// Issue a spl_token `Burn` instruction.
#[inline(always)]
fn spl_token_burn(params: TokenBurnParams<'_, '_>) -> ProgramResult {
    let TokenBurnParams {
        mint,
        source,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;
    let result = invoke_optionally_signed(
        &spl_token::instruction::burn(
            token_program.key,
            source.key,
            mint.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, mint, authority, token_program],
        authority_signer_seeds,
    );
    result.map_err(|_| LendingError::TokenBurnFailed.into())
}

/// validates reserve configs
#[inline(always)]
fn validate_reserve_config(config: ReserveConfig) -> ProgramResult {
    if config.optimal_utilization_rate > 100 {
        msg!("Optimal utilization rate must be in range [0, 100]");
        return Err(LendingError::InvalidConfig.into());
    }
    if config.loan_to_value_ratio >= 100 {
        msg!("Loan to value ratio must be in range [0, 100)");
        return Err(LendingError::InvalidConfig.into());
    }
    if config.liquidation_bonus > 100 {
        msg!("Liquidation bonus must be in range [0, 100]");
        return Err(LendingError::InvalidConfig.into());
    }
    if config.liquidation_threshold < config.loan_to_value_ratio
        || config.liquidation_threshold > 100
    {
        msg!("Liquidation threshold must be in range [LTV, 100]");
        return Err(LendingError::InvalidConfig.into());
    }
    if config.optimal_borrow_rate < config.min_borrow_rate {
        msg!("Optimal borrow rate must be >= min borrow rate");
        return Err(LendingError::InvalidConfig.into());
    }
    if config.optimal_borrow_rate > config.max_borrow_rate {
        msg!("Optimal borrow rate must be <= max borrow rate");
        return Err(LendingError::InvalidConfig.into());
    }
    if config.fees.borrow_fee_wad >= WAD {
        msg!("Borrow fee must be in range [0, 1_000_000_000_000_000_000)");
        return Err(LendingError::InvalidConfig.into());
    }
    if config.fees.host_fee_percentage > 100 {
        msg!("Host fee percentage must be in range [0, 100]");
        return Err(LendingError::InvalidConfig.into());
    }
    if config.protocol_liquidation_fee > 100 {
        msg!("Protocol liquidation fee must be in range [0, 100]");
        return Err(LendingError::InvalidConfig.into());
    }
    if config.protocol_take_rate > 100 {
        msg!("Protocol take rate must be in range [0, 100]");
        return Err(LendingError::InvalidConfig.into());
    }
    Ok(())
}

/// validates pyth AccountInfos
#[inline(always)]
fn validate_pyth_keys(
    lending_market: &LendingMarket,
    pyth_product_info: &AccountInfo,
    pyth_price_info: &AccountInfo,
) -> ProgramResult {
    if *pyth_price_info.key == solend_program::NULL_PUBKEY {
        return Ok(());
    }
    if &lending_market.oracle_program_id != pyth_product_info.owner {
        msg!("Pyth product account provided is not owned by the lending market oracle program");
        return Err(LendingError::InvalidOracleConfig.into());
    }
    if &lending_market.oracle_program_id != pyth_price_info.owner {
        msg!("Pyth price account provided is not owned by the lending market oracle program");
        return Err(LendingError::InvalidOracleConfig.into());
    }

    let pyth_product_data = pyth_product_info.try_borrow_data()?;
    let pyth_product = pyth_sdk_solana::state::load_product_account(&pyth_product_data)?;

    if &pyth_product.px_acc != pyth_price_info.key {
        msg!("Pyth product price account does not match the Pyth price provided");
        return Err(LendingError::InvalidOracleConfig.into());
    }

    let quote_currency = get_pyth_product_quote_currency(pyth_product)?;
    if lending_market.quote_currency != quote_currency {
        msg!("Lending market quote currency does not match the oracle quote currency");
        return Err(LendingError::InvalidOracleConfig.into());
    }
    Ok(())
}

/// validates switchboard AccountInfo
fn validate_switchboard_keys(
    lending_market: &LendingMarket,
    switchboard_feed_info: &AccountInfo,
) -> ProgramResult {
    if *switchboard_feed_info.key == solend_program::NULL_PUBKEY {
        return Ok(());
    }
    if switchboard_feed_info.owner != &lending_market.switchboard_oracle_program_id
        && switchboard_feed_info.owner != &switchboard_v2_mainnet::id()
        && switchboard_feed_info.owner != &switchboard_v2_devnet::id()
    {
        msg!("Switchboard account provided is not owned by the switchboard oracle program");
        return Err(LendingError::InvalidOracleConfig.into());
    }
    Ok(())
}

fn is_cpi_call(
    program_id: &Pubkey,
    current_index: usize,
    sysvar_info: &AccountInfo,
) -> Result<bool, ProgramError> {
    // say the tx looks like:
    // ix 0
    //   - ix a
    //   - ix b
    //   - ix c
    // ix 1
    // and we call "load_current_index_checked" from b, we will get 0. And when we
    // load_instruction_at_checked(0), we will get ix 0.
    // tldr; instructions sysvar only stores top-level instructions, never CPI instructions.
    let current_ixn = load_instruction_at_checked(current_index, sysvar_info)?;

    // the current ixn must match the flash_* ix. otherwise, it's a CPI. Comparing program_ids is a
    // cheaper way of verifying this property, bc token-lending doesn't allow re-entrancy anywhere.
    if *program_id != current_ixn.program_id {
        return Ok(true);
    }

    if get_stack_height() > TRANSACTION_LEVEL_STACK_HEIGHT {
        return Ok(true);
    }

    Ok(false)
}

struct TokenInitializeMintParams<'a: 'b, 'b> {
    mint: AccountInfo<'a>,
    rent: AccountInfo<'a>,
    authority: &'b Pubkey,
    decimals: u8,
    token_program: AccountInfo<'a>,
}

struct TokenInitializeAccountParams<'a> {
    account: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    owner: AccountInfo<'a>,
    rent: AccountInfo<'a>,
    token_program: AccountInfo<'a>,
}

struct TokenTransferParams<'a: 'b, 'b> {
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    amount: u64,
    authority: AccountInfo<'a>,
    authority_signer_seeds: &'b [&'b [u8]],
    token_program: AccountInfo<'a>,
}

struct TokenMintToParams<'a: 'b, 'b> {
    mint: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    amount: u64,
    authority: AccountInfo<'a>,
    authority_signer_seeds: &'b [&'b [u8]],
    token_program: AccountInfo<'a>,
}

struct TokenBurnParams<'a: 'b, 'b> {
    mint: AccountInfo<'a>,
    source: AccountInfo<'a>,
    amount: u64,
    authority: AccountInfo<'a>,
    authority_signer_seeds: &'b [&'b [u8]],
    token_program: AccountInfo<'a>,
}
