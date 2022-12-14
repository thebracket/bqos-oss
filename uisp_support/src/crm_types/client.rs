use super::attribute::Attribute;
use super::bank_account::BankAccount;
use super::contact::Contact;
use super::tag::Tag;
use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Client {
    pub id: Option<usize>,
    pub userIdent: Option<String>,
    pub previousIsp: Option<String>,
    pub isLead: Option<bool>,
    pub clientType: Option<usize>,
    pub companyName: Option<String>,
    pub companyRegistrationNumber: Option<String>,
    pub companyTaxId: Option<String>,
    pub companyWebsite: Option<String>,
    pub companyContactFirstName: Option<String>,
    pub companyContactLastName: Option<String>,
    pub firstName: Option<String>,
    pub lastName: Option<String>,
    pub street1: Option<String>,
    pub street2: Option<String>,
    pub city: Option<String>,
    pub countryId: Option<usize>,
    pub stateId: Option<usize>,
    pub zipCode: Option<String>,
    pub invoiceStreet1: Option<String>,
    pub invoiceStreet2: Option<String>,
    pub invoiceCity: Option<String>,
    pub invoiceStateId: Option<usize>,
    pub invoiceCountryId: Option<usize>,
    pub invoiceZipCode: Option<String>,
    pub invoiceAddressSameAsContact: Option<bool>,
    pub note: Option<String>,
    pub sendInvoiceByPost: Option<bool>,
    pub invoiceMaturityDays: Option<usize>,
    pub stopServiceDue: Option<bool>,
    pub stopServiceDueDays: Option<usize>,
    pub organizationId: Option<usize>,
    pub tax1Id: Option<usize>,
    pub tax2Id: Option<usize>,
    pub tax3Id: Option<usize>,
    pub registrationDate: Option<String>,
    pub username: Option<String>,
    pub avatarColor: Option<String>,
    pub addressGpsLat: Option<f32>,
    pub addressGpsLon: Option<f32>,
    pub generateProformaInvoices: Option<bool>,
    pub isActive: Option<bool>,
    pub contacts: Option<Vec<Contact>>,
    pub attributes: Option<Vec<Attribute>>,
    pub accountBalance: Option<f32>,
    pub accountCredit: Option<f32>,
    pub accountOutstanding: Option<f32>,
    pub currencyCode: Option<String>,
    pub organizationName: Option<String>,
    pub bankAccounts: Option<Vec<BankAccount>>,
    pub tags: Option<Vec<Tag>>,
    pub invitationEmailSentDate: Option<String>,
    pub isArchived: Option<bool>,
    pub usesProforma: Option<bool>,
}
