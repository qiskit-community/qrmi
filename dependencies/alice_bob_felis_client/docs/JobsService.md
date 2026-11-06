# \JobsService

All URIs are relative to */external*

Method | HTTP request | Description
------------- | ------------- | -------------
[**cancel_job**](JobsService.md#cancel_job) | **DELETE** /v1/jobs/{job_id} | Cancel Job
[**create_job**](JobsService.md#create_job) | **POST** /v1/jobs/ | Create Job
[**download_input**](JobsService.md#download_input) | **GET** /v1/jobs/{job_id}/input | Download Input
[**download_memory**](JobsService.md#download_memory) | **GET** /v1/jobs/{job_id}/memory | Download Memory
[**download_output**](JobsService.md#download_output) | **GET** /v1/jobs/{job_id}/output | Download Output
[**download_transpiled**](JobsService.md#download_transpiled) | **GET** /v1/jobs/{job_id}/transpiled | Download Transpiled
[**get_job**](JobsService.md#get_job) | **GET** /v1/jobs/{job_id} | Get Job
[**get_job_metrics**](JobsService.md#get_job_metrics) | **GET** /v1/jobs/{job_id}/metrics | Get Job Metrics
[**list_jobs**](JobsService.md#list_jobs) | **GET** /v1/jobs/ | List Jobs
[**upload_input**](JobsService.md#upload_input) | **POST** /v1/jobs/{job_id}/input | Upload Input



## cancel_job

> models::ExternalJob cancel_job(job_id, authorization)
Cancel Job

Cancel a given active job.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **uuid::Uuid** |  | [required] |
**authorization** | Option<**String**> |  |  |

### Return type

[**models::ExternalJob**](ExternalJob.md)

### Authorization

[basicAuth](../README.md#basicAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_job

> models::ExternalJob create_job(create_external_job, authorization)
Create Job

Create a new job to be run on a specific target. The job's code needs to be uploaded separately using \"Upload Input\" to start running.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_external_job** | [**CreateExternalJob**](CreateExternalJob.md) |  | [required] |
**authorization** | Option<**String**> |  |  |

### Return type

[**models::ExternalJob**](ExternalJob.md)

### Authorization

[basicAuth](../README.md#basicAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## download_input

> serde_json::Value download_input(job_id, authorization)
Download Input

Return the raw input QIR code of a given job as it was submitted to the API.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **uuid::Uuid** |  | [required] |
**authorization** | Option<**String**> |  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[basicAuth](../README.md#basicAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## download_memory

> serde_json::Value download_memory(job_id, authorization)
Download Memory

Return the results of a given job as a list of measurements of each shot

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **uuid::Uuid** |  | [required] |
**authorization** | Option<**String**> |  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[basicAuth](../README.md#basicAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## download_output

> String download_output(job_id, authorization)
Download Output

Return the results of a given job as a list of measurements with their number of occurrences.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **uuid::Uuid** |  | [required] |
**authorization** | Option<**String**> |  |  |

### Return type

**String**

### Authorization

[basicAuth](../README.md#basicAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/plain, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## download_transpiled

> serde_json::Value download_transpiled(job_id, authorization)
Download Transpiled

Return the QIR code of a given job transpiled for the specificities of the target's gate set.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **uuid::Uuid** |  | [required] |
**authorization** | Option<**String**> |  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[basicAuth](../README.md#basicAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job

> models::ExternalJob get_job(job_id, authorization)
Get Job

Get details on a given job.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **uuid::Uuid** |  | [required] |
**authorization** | Option<**String**> |  |  |

### Return type

[**models::ExternalJob**](ExternalJob.md)

### Authorization

[basicAuth](../README.md#basicAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_metrics

> models::ExternalJobMetrics get_job_metrics(job_id, authorization)
Get Job Metrics

Return recorded metrics about the job: > `qpu_duration_ns`: duration recorded executing a circuit on a QPU, in nanoseconds. > `simulation_duration_ns`: duration recorded simulating a circuit, in nanoseconds.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **uuid::Uuid** |  | [required] |
**authorization** | Option<**String**> |  |  |

### Return type

[**models::ExternalJobMetrics**](ExternalJobMetrics.md)

### Authorization

[basicAuth](../README.md#basicAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_jobs

> Vec<models::ExternalJob> list_jobs(page, limit, authorization)
List Jobs

Return all active and completed jobs associated with the authenticated user.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> |  |  |[default to 1]
**limit** | Option<**i32**> |  |  |[default to 100]
**authorization** | Option<**String**> |  |  |

### Return type

[**Vec<models::ExternalJob>**](ExternalJob.md)

### Authorization

[basicAuth](../README.md#basicAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## upload_input

> serde_json::Value upload_input(job_id, input, authorization)
Upload Input

Upload the input code for a job in QIR format to trigger its execution (provided that the target is available).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **uuid::Uuid** |  | [required] |
**input** | **std::path::PathBuf** |  | [required] |
**authorization** | Option<**String**> |  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[basicAuth](../README.md#basicAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

