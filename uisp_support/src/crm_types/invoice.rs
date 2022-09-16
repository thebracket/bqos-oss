use crate::crm_types::Attribute;
use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Invoice {
    pub clientId: Option<usize>,
    pub number: Option<String>,
    pub createdDate: Option<String>,
    pub emailSentDate: Option<String>,
    pub maturityDays: Option<usize>,
    pub notes: Option<String>,
    pub discount: Option<f32>,
    pub discountLabel: Option<String>,
    pub adminNotes: Option<String>,
    pub invoiceTemplateId: Option<usize>,
    pub proformaInvoiceId: Option<usize>,
    pub organizationName: Option<String>,
    pub organizationRegistrationNumber: Option<String>,
    pub organizationTaxId: Option<String>,
    pub organizationStreet1: Option<String>,
    pub organizationStreet2: Option<String>,
    pub organizationCity: Option<String>,
    pub organizationCountryId: Option<usize>,
    pub organizationStateId: Option<usize>,
    pub organizationZipCode: Option<String>,
    pub organizationBankAccountName: Option<String>,
    pub organizationBankAccountField1: Option<String>,
    pub organizationBankAccountField2: Option<String>,
    pub clientFirstName: Option<String>,
    pub clientLastName: Option<String>,
    pub clientCompanyName: Option<String>,
    pub clientCompanyRegistrationNumber: Option<String>,
    pub clientCompanyTaxId: Option<String>,
    pub clientStreet1: Option<String>,
    pub clientStreet2: Option<String>,
    pub clientCity: Option<String>,
    pub clientCountryId: Option<usize>,
    pub clientStateId: Option<usize>,
    pub clientZipCode: Option<String>,
    pub proforma: Option<bool>,
    pub dueDate: Option<String>,
    pub taxableSupplyDate: Option<String>,
    pub items: Option<Vec<InvoiceItem>>,
    pub subtotal: Option<f32>,
    pub taxes: Option<Vec<InvoiceTaxes>>,
    pub total: Option<f32>,
    pub amountPaid: Option<f32>,
    pub amountToPay: Option<f32>,
    pub totalUntaxed: Option<f32>,
    pub totalDiscount: Option<f32>,
    pub totalTaxAmount: Option<f32>,
    pub currencyCode: Option<String>,
    pub status: Option<usize>,
    pub paymentCovers: Option<Vec<InvoicePaymentCovers>>,
    pub uncollectible: Option<bool>,
    pub generatedInvoiceId: Option<usize>,
    pub attributes: Option<Vec<Attribute>>,
    pub id: Option<usize>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct InvoiceItem {
    pub label: Option<String>,
    pub price: Option<f32>,
    pub quantity: Option<f32>,
    pub unit: Option<String>,
    pub tax1Id: Option<usize>,
    pub tax2Id: Option<usize>,
    pub tax3Id: Option<usize>,
    pub id: Option<usize>,
    pub serviceId: Option<usize>,
    pub serviceSurchargeId: Option<usize>,
    pub productId: Option<usize>,
    pub total: Option<f32>,
    #[serde(alias = "type")]
    pub item_type: Option<String>,
    pub discount_price: Option<f32>,
    // pub discountQuantity
    // pub discountTotal
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct InvoiceTaxes {
    pub name: Option<String>,
    pub totalValue: Option<f32>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct InvoicePaymentCovers {
    pub id: Option<usize>,
    pub paymentId: Option<usize>,
    pub invoiceId: Option<usize>,
    pub refundId: Option<usize>,
    pub amount: Option<f32>,
}
