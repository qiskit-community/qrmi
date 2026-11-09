# \HealthService

All URIs are relative to */external*

Method | HTTP request | Description
------------- | ------------- | -------------
[**check_health**](HealthService.md#check_health) | **GET** /v1/health/ | Check Health



## check_health

> String check_health(authorization)
Check Health

Return \"OK\" if the Felis Cloud API is up and running.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**authorization** | Option<**String**> |  |  |

### Return type

**String**

### Authorization

[basicAuth](../README.md#basicAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/plain, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

