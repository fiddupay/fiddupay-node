# Complete SDK Method Index

## AddressOnly (`fiddupay.addressOnly`)
- `fiddupay.addressOnly.create()`
- `fiddupay.addressOnly.retrieve()`
- `fiddupay.addressOnly.listCurrencies()`
- `fiddupay.addressOnly.getStats()`
- `fiddupay.addressOnly.getHealth()`
- `fiddupay.addressOnly.getFeeSetting()`
- `fiddupay.addressOnly.setFeeSetting()`

## Analytics (`fiddupay.analytics`)
- `fiddupay.analytics.retrieve()`
- `fiddupay.analytics.export()`
- `fiddupay.analytics.getUnifiedTransactions()`

## AuditLogs (`fiddupay.auditLogs`)
- `fiddupay.auditLogs.list()`

## Balances (`fiddupay.balances`)
- `fiddupay.balances.get()`
- `fiddupay.balances.getHistory()`

## Contact (`fiddupay.contact`)
- `fiddupay.contact.submit()`

## Customers (`fiddupay.customers`)
- `fiddupay.customers.register()`
- `fiddupay.customers.list()`
- `fiddupay.customers.getBalances()`
- `fiddupay.customers.getWallets()`
- `fiddupay.customers.createWallets()` (Provision Wallets)
- `fiddupay.customers.updateStatus()`
- `fiddupay.customers.updatePermissions()`
- `fiddupay.customers.sweep({ sweep_mode: 'ALL' })` — sweep all assets to Master Wallet
- `fiddupay.customers.sweep({ sweep_mode: 'NATIVE_ONLY' })` — sweep native coins only (ETH, BNB, SOL…)
- `fiddupay.customers.sweep({ sweep_mode: 'STABLE_ONLY' })` — sweep stablecoins only (USDT…)
- `fiddupay.customers.sweep({ sweep_mode: 'SPECIFIC', crypto_types: [...] })` — sweep a specific asset
- `fiddupay.customers.payMerchant()` — lock customer funds into merchant reserved balance
- `fiddupay.customers.getDepositAddress()`
- `fiddupay.customers.getTransactions()`
- `fiddupay.customers.deactivate()`
- `fiddupay.customers.bulkProvision()` — provision wallets for multiple customers at once
- `fiddupay.customers.getSummary()` — retrieve aggregate platform customer statistics
- `fiddupay.customers.verifyAndRepairWallets()` — verify and auto-repair customer deposit wallets
- `fiddupay.customers.lookupAddress()` — reverse lookup on-chain wallet address to customer
- `fiddupay.customers.auditWallets()` — generate active vs historical wallet audit report
- `fiddupay.customers.getUnsweptAssetsSummary()` — aggregate summary of unswept customer assets
- `fiddupay.customers.batchSweep()` — single-click batch sweep of unswept customer assets

## Invoices (`fiddupay.invoices`)
- `fiddupay.invoices.create()`
- `fiddupay.invoices.list()`
- `fiddupay.invoices.retrieve()`

## Merchants (`fiddupay.merchants`)
- `fiddupay.merchants.register()`
- `fiddupay.merchants.login()`
- `fiddupay.merchants.logout()`
- `fiddupay.merchants.retrieve()` (Profile)
- `fiddupay.merchants.getStatus()`
- `fiddupay.merchants.getReadiness()` (aliasing `getStatus`)
- `fiddupay.merchants.switchEnvironment()`
- `fiddupay.merchants.claimUsername()`
- `fiddupay.merchants.updateKycDraft()`
- `fiddupay.merchants.generateApiKey()`
- `fiddupay.merchants.rotateApiKey()`
- `fiddupay.merchants.getFeeSetting()`
- `fiddupay.merchants.updateSettings()`
- `fiddupay.merchants.getSettings()`
- `fiddupay.merchants.sendTestWebhook()`
- `fiddupay.merchants.getIpWhitelist()`
- `fiddupay.merchants.getBalance()`
- `fiddupay.merchants.getAuditLogs()`
- `fiddupay.merchants.getBalanceHistory()`

## Notifications (`fiddupay.notifications`)
- `fiddupay.notifications.list()`
- `fiddupay.notifications.markRead()`
- `fiddupay.notifications.delete()`

## Payments (`fiddupay.payments`)
- `fiddupay.payments.create()`
- `fiddupay.payments.list()`
- `fiddupay.payments.retrieve()`
- `fiddupay.payments.cancel()`
- `fiddupay.payments.verify()`
- `fiddupay.payments.finalizeSelection()`

## Refunds (`fiddupay.refunds`)
- `fiddupay.refunds.create()`
- `fiddupay.refunds.list()`
- `fiddupay.refunds.retrieve()`
- `fiddupay.refunds.complete()`

## Security (`fiddupay.security`)
- `fiddupay.security.getEvents()`
- `fiddupay.security.getAlerts()`
- `fiddupay.security.acknowledgeAlert()`
- `fiddupay.security.getBalanceAlerts()`
- `fiddupay.security.resolveBalanceAlert()`
- `fiddupay.security.toggleWalletLock()`
- `fiddupay.security.toggleCustomerWalletLock()`
- `fiddupay.security.setTransactionPin()`
- `fiddupay.security.verifyTransactionPin()`
- `fiddupay.security.gasCheck()`

## Transactions (`fiddupay.transactions`)
- `fiddupay.transactions.list()`

## Wallets (`fiddupay.wallets`)
- `fiddupay.wallets.setup()`
- `fiddupay.wallets.getConfigurations()`
- `fiddupay.wallets.getBalances()`
- `fiddupay.wallets.revoke()`
- `fiddupay.wallets.checkGasRequirements()`
- `fiddupay.wallets.getGasEstimates()`
- `fiddupay.wallets.checkWithdrawalCapability()`

## Webhooks (`fiddupay.webhooks` / `Webhooks`)
- `fiddupay.webhooks.listDeliveries()` — list historic webhook delivery attempts
- `fiddupay.webhooks.retryDelivery()` — re-queue webhook delivery for immediate retry
- `Webhooks.constructEvent()` (Static Utility & Instance Method)
- `Webhooks.verifySignature()` (Static Utility & Instance Method)
- `Webhooks.generateSignature()` (Static Utility & Instance Method)

## Withdrawals (`fiddupay.withdrawals`)
- `fiddupay.withdrawals.create()`
- `fiddupay.withdrawals.list()`
- `fiddupay.withdrawals.get()`
- `fiddupay.withdrawals.cancel()`
- `fiddupay.withdrawals.process()`
- `fiddupay.withdrawals.validateGas()`
- `fiddupay.withdrawals.getGasEstimates()`
- `fiddupay.withdrawals.checkCapability()`
