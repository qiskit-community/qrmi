# \JobsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**cancel_job_v1**](JobsApi.md#cancel_job_v1) | **POST** /api/v1/jobs/{job_id}/cancel | Cancel job
[**delete_job_v1**](JobsApi.md#delete_job_v1) | **DELETE** /api/v1/jobs/{job_id} | Delete job
[**get_job_payload_v1**](JobsApi.md#get_job_payload_v1) | **GET** /api/v1/jobs/{job_id}/payload | Get job payload
[**get_job_v1**](JobsApi.md#get_job_v1) | **GET** /api/v1/jobs/{job_id} | Get job
[**job_get_artifacts**](JobsApi.md#job_get_artifacts) | **GET** /api/v1/jobs/{job_id}/artifacts/{artifact_type} | Get job artifact
[**job_submit**](JobsApi.md#job_submit) | **POST** /api/v1/jobs/{qc}/{job_type} | Submit job



## cancel_job_v1

> models::IqmServerJob cancel_job_v1(job_id)
Cancel job

 Cancels a job identified by its job ID.  If the job is currently executing, the system interrupts its execution. Only artifacts produced before the interruption remain available for interrupted jobs. Jobs that have already reached a terminal status cannot be cancelled. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **uuid::Uuid** | Job ID | [required] |

### Return type

[**models::IqmServerJob**](iqm.server.Job.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_job_v1

> delete_job_v1(job_id)
Delete job

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **uuid::Uuid** | Job ID | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_payload_v1

> get_job_payload_v1(job_id)
Get job payload

 Retrieves the original input payload of a job identified by its job ID.  The payload is returned exactly as it was at the time of submission, without any modifications. If the job type's payload schema changes in the future, those changes are not applied to previously submitted job payloads.  The type of the submitted job is provided in the `IQM-Job-Type` response header.  Payloads are stored in a compressed format. For best performance, include `Accept-Encoding: gzip, deflate` in the request headers so that IQM Server can serve the stored data directly. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **uuid::Uuid** | Job ID | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_v1

> models::IqmServerJob get_job_v1(job_id, wait_until_terminal, timeout_seconds)
Get job

 Retrieves the job status and state metadata of a job identified by its job ID.  Note that this response does not include raw job artifact data (e.g., measurement results). To access artifact data, use the **Get job artifact** endpoint once the artifacts become available for the job.  When `wait_until_terminal=true` is supplied the server holds the request open for up to `timeout_seconds` seconds (default 10, max 30) while it waits for the job to reach a terminal API status (one of `completed`, `failed`, or `cancelled`). If the timeout elapses first, the server responds with the current job state and sets the `IQM-Long-Poll-Timed-Out: true` response header so clients can reissue the request without parsing the body. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **uuid::Uuid** | Job ID | [required] |
**wait_until_terminal** | Option<**bool**> | When `true`, the server holds the request open until the job reaches a terminal API status — one of `completed`, `failed`, or `cancelled` — or the timeout elapses. If the job is already terminal the server responds immediately. When omitted or `false` the endpoint returns the current job state without waiting. |  |[default to false]
**timeout_seconds** | Option<**i32**> | Maximum number of seconds the server will hold the request open when `wait_until_terminal=true`. Must be between 0 and 30 (inclusive). Defaults to 10. Ignored when `wait_until_terminal` is not set. When the timeout elapses before the job reaches a terminal API status, the server returns the current job state and sets the `IQM-Long-Poll-Timed-Out: true` response header. |  |[default to 10]

### Return type

[**models::IqmServerJob**](iqm.server.Job.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## job_get_artifacts

> job_get_artifacts(job_id, artifact_type)
Get job artifact

 Retrieves the raw job output artifact data for a job identified by its job ID and artifact type.  The available job artifacts depend on the job type and the Station Control version of the quantum computer. Refer to the auto-generated, quantum computer-specific API documentation for details on supported artifacts and their corresponding data formats.  The list of available artifacts for each job is included in the **Get Job** endpoint response.  Artifacts are stored in a compressed format. For best performance, include `Accept-Encoding: gzip, deflate` in the request headers so that IQM Server can serve the stored data directly. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **uuid::Uuid** | Job ID | [required] |
**artifact_type** | **String** | Job artifact type | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## job_submit

> models::IqmServerJob job_submit(qc, job_type, use_timeslot, tag, body)
Submit job

 Submits a new job for execution. The required payload structure depends on the job type being submitted. Refer to the auto-generated, quantum computer-specific API documentation for details on supported job types and their corresponding payload formats. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**qc** | **String** | Quantum computer id or alias | [required] |
**job_type** | **String** | Submitted job type. Allowed values: `circuit`, `sweep` | [required] |
**use_timeslot** | Option<**bool**> | Set `use_timeslot=true` to submit the job to the timeslot queue instead of the default FIFO queue. Jobs submitted with this flag will be processed during the next account timeslot. |  |
**tag** | Option<**String**> | Optional user-defined tag associated with the job. Can be used for external identification or filtering jobs. Max length is 50 characters. |  |
**body** | Option<**serde_json::Value**> | Submitted job payload. |  |

### Return type

[**models::IqmServerJob**](iqm.server.Job.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

