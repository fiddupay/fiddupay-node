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
- `fiddupay.customers.withdraw()`
- `fiddupay.customers.sweep()`
- `fiddupay.customers.deactivate()`
- `fiddupay.customers.getTransactions()`
- `fiddupay.customers.getDepositAddress()`
- `fiddupay.customers.payMerchant()`

## Invoices (`fiddupay.invoices`)
- `fiddupay.invoices.create()`
- `fiddupay.invoices.list()`
- `fiddupay.invoices.retrieve()`

## Merchants (`fiddupay.merchants`)
- `fiddupay.merchants.register()`
- `fiddupay.merchants.retrieve()` (Profile)
- `fiddupay.merchants.getStatus()`
- `fiddupay.merchants.switchEnvironment()`
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

## Sandbox (`fiddupay.sandbox`)
- `fiddupay.sandbox.simulatePayment()`

## Security (`fiddupay.security`)
- `fiddupay.security.getEvents()`
- `fiddupay.security.getAlerts()`
- `fiddupay.security.acknowledgeAlert()`
- `fiddupay.security.getBalanceAlerts()`
- `fiddupay.security.resolveBalanceAlert()`
- `fiddupay.security.checkGasBalances()`
- `fiddupay.security.getSettings()`
- `fiddupay.security.toggleWalletLock()`
- `fiddupay.security.toggleCustomerWalletLock()`

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

## Withdrawals (`fiddupay.withdrawals`)
- `fiddupay.withdrawals.create()`
- `fiddupay.withdrawals.list()`
- `fiddupay.withdrawals.get()`
- `fiddupay.withdrawals.cancel()`
- `fiddupay.withdrawals.process()`
- `fiddupay.withdrawals.validateGas()`
- `fiddupay.withdrawals.getGasEstimates()`
- `fiddupay.withdrawals.checkCapability()`

## Webhooks (`Webhooks`)
- `Webhooks.constructEvent()` (Static Utility)
