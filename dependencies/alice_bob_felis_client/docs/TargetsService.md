# \TargetsService

All URIs are relative to */external*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_target_availabilities**](TargetsService.md#list_target_availabilities) | **GET** /v1/targets/{target}/availabilities | List Target Availabilities
[**list_targets**](TargetsService.md#list_targets) | **GET** /v1/targets/ | List Targets
[**target_status**](TargetsService.md#target_status) | **GET** /v1/targets/{target}/health | Target Status



## list_target_availabilities

> Vec<models::TargetAvailability> list_target_availabilities(target, limit, offset)
List Target Availabilities

Get the latest entries of a target's planned availabilities.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**target** | **String** |  | [required] |
**limit** | Option<**i32**> |  |  |[default to 100]
**offset** | Option<**i32**> |  |  |[default to 0]

### Return type

[**Vec<models::TargetAvailability>**](TargetAvailability.md)

### Authorization

[basicAuth](../README.md#basicAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_targets

> Vec<models::TargetConfiguration> list_targets()
List Targets

Return the list of all targets to execute the jobs on. These targets can be emulators or real quantum hardware.

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::TargetConfiguration>**](TargetConfiguration.md)

### Authorization

[basicAuth](../README.md#basicAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## target_status

> models::TargetStatus target_status(target)
Target Status

Return the current status of a target: \"OK\" if the target is enabled and in a working state. \"NOK\" if the target is enabled but monitored to be in a bad state. \"OFF\" if the target is disabled at this given time.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**target** | **String** |  | [required] |

### Return type

[**models::TargetStatus**](TargetStatus.md)

### Authorization

[basicAuth](../README.md#basicAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

