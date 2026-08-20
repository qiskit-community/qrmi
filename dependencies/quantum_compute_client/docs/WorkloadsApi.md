# \WorkloadsApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**find_instance_workloads**](WorkloadsApi.md#find_instance_workloads) | **GET** /v1/workloads | List user instance workloads



## find_instance_workloads

> models::FindInstanceWorkloads200Response find_instance_workloads(ibm_api_version, sort, limit, previous, next, backend, search, status, mode, created_after, created_before, tags)
List user instance workloads

List user instance workloads

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]
**sort** | Option<**String**> | Field to sort the workloads by. A `-` prefix indicates descending sort order. |  |[default to createdAt]
**limit** | Option<**f64**> | Number of workloads to return at a time |  |[default to 10]
**previous** | Option<**String**> | Cursor to previous workloads result page |  |
**next** | Option<**String**> | Cursor to next workloads result page |  |
**backend** | Option<**String**> | Backend name |  |
**search** | Option<**String**> | Optional search string, used to filter workloads by id or tags |  |
**status** | Option<[**Vec<String>**](String.md)> | Status type to filter workloads by. It can be pending, in_progress, failed, completed or canceled. |  |
**mode** | Option<**String**> | Workload mode: job, session or batch |  |
**created_after** | Option<**String**> | Filter jobs and session created after this date |  |
**created_before** | Option<**String**> | Filter jobs and session created before this date |  |
**tags** | Option<[**Vec<String>**](String.md)> | Optional array of tags for the workloads |  |

### Return type

[**models::FindInstanceWorkloads200Response**](find_instance_workloads_200_response.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

