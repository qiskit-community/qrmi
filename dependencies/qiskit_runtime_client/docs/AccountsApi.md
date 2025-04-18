# \AccountsApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_account_config**](AccountsApi.md#get_account_config) | **GET** /v1/accounts/{id} | Get account configuration



## get_account_config

> models::GetAccountConfig200Response get_account_config(id, ibm_api_version, plan_id)
Get account configuration

Get the current account information. If no account information is found, returns the default configuration.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Account id (without `a/` prefix) | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]
**plan_id** | Option<**String**> | Obtain the account configuration only for the specified plan |  |

### Return type

[**models::GetAccountConfig200Response**](get_account_config_200_response.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

