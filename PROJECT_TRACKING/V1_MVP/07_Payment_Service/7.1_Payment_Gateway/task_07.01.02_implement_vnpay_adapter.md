# Task: Implement VNPay Payment Gateway Adapter

**Task ID:** V1_MVP/07_Payment_Service/7.1_Payment_Gateway/task_07.01.02_implement_vnpay_adapter.md
**Version:** V1_MVP
**Phase:** 07_Payment_Service
**Module:** 7.1_Payment_Gateway
**Priority:** High
**Status:** Todo
**Assignee:**
**Created Date:** 2025-01-21
**Last Updated:** 2025-01-21

## Detailed Description:
Implement VNPay payment gateway adapter with full API integration including payment creation, confirmation, and webhook handling for Vietnamese market.

## Specific Sub-tasks:
- [ ] 1. Create `VNPayAdapter` struct implementing PaymentAdapter trait
- [ ] 2. Implement VNPay payment URL generation with secure hash
- [ ] 3. Handle VNPay return URL and IPN (Instant Payment Notification)
- [ ] 4. Implement payment status checking and verification
- [ ] 5. Add support for VNPay payment methods (ATM, credit card, e-wallet)
- [ ] 6. Implement refund processing for VNPay transactions
- [ ] 7. Handle VNPay webhook notifications and signature verification
- [ ] 8. Add comprehensive error handling for VNPay-specific errors
- [ ] 9. Implement retry logic for failed payment operations
- [ ] 10. Create payment request/response mapping

## Acceptance Criteria:
- [ ] VNPayAdapter fully implements PaymentAdapter trait
- [ ] Payment URL generation working with proper security hash
- [ ] Return URL and IPN handling operational
- [ ] Payment status verification implemented
- [ ] Multiple payment methods supported
- [ ] Refund processing functional
- [ ] Webhook signature verification working
- [ ] Error handling comprehensive for VNPay APIs
- [ ] Retry logic for failed operations
- [ ] Comprehensive test coverage for VNPay integration

## Dependencies:
- V1_MVP/07_Payment_Service/7.1_Payment_Gateway/task_07.01.01_create_payment_adapter_trait.md

## Related Documents:
- `services/payment_service/core/src/adapters/vnpay.rs` (file to be created)
- `services/payment_service/core/src/adapters/vnpay/client.rs` (file to be created)
- `services/payment_service/core/src/adapters/vnpay/models.rs` (file to be created)

## Notes / Discussion:
---
* VNPay requires TMN_CODE and SECRET_KEY for authentication
* Implement proper hash calculation for payment requests
* Handle VNPay's specific response format and error codes
* Support Vietnamese language and currency (VND)
* Monitor VNPay's API rate limits and implement backoff strategies

## AI Agent Log:
---
* (Log will be automatically updated by AI agent when starting and executing task)