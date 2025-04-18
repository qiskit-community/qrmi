# \JobsApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**cancel_job_jid**](JobsApi.md#cancel_job_jid) | **POST** /v1/jobs/{id}/cancel | Cancel a job
[**create_job**](JobsApi.md#create_job) | **POST** /v1/jobs | Run a job
[**delete_job_jid**](JobsApi.md#delete_job_jid) | **DELETE** /v1/jobs/{id} | Delete a job
[**get_interim_results_jid**](JobsApi.md#get_interim_results_jid) | **GET** /v1/jobs/{id}/interim_results | List job interim results
[**get_job_details_jid**](JobsApi.md#get_job_details_jid) | **GET** /v1/jobs/{id} | List job details
[**get_job_metrics_jid**](JobsApi.md#get_job_metrics_jid) | **GET** /v1/jobs/{id}/metrics | Get job metrics
[**get_job_results_jid**](JobsApi.md#get_job_results_jid) | **GET** /v1/jobs/{id}/results | List job results
[**get_jog_logs_jid**](JobsApi.md#get_jog_logs_jid) | **GET** /v1/jobs/{id}/logs | List job logs
[**get_transpiled_circuits_jid**](JobsApi.md#get_transpiled_circuits_jid) | **GET** /v1/jobs/{id}/transpiled_circuits | Get job transpiled circuits
[**list_jobs**](JobsApi.md#list_jobs) | **GET** /v1/jobs | List jobs
[**replace_job_tags**](JobsApi.md#replace_job_tags) | **PUT** /v1/jobs/{id}/tags | Replace job tags



## cancel_job_jid

> cancel_job_jid(id, parent_job_id, ibm_api_version)
Cancel a job

Cancels the specified job.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | A job ID | [required] |
**parent_job_id** | Option<**String**> | Parent job ID |  |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]

### Return type

 (empty response body)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_job

> models::CreateJob200Response create_job(ibm_api_version, parent_job_id, create_job_request)
Run a job

Invoke a Qiskit Runtime primitive. Note the returned job ID.  You will use it to check the job's status and review results. This request is rate limited to 5 jobs per minute per user.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]
**parent_job_id** | Option<**String**> | Parent job ID |  |
**create_job_request** | Option<[**CreateJobRequest**](CreateJobRequest.md)> |  |  |

### Return type

[**models::CreateJob200Response**](create_job_200_response.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_job_jid

> delete_job_jid(id, ibm_api_version)
Delete a job

Delete the specified job and its associated data. Job must be in a terminal state.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Identifier of an existing job | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]

### Return type

 (empty response body)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_interim_results_jid

> String get_interim_results_jid(id, ibm_api_version)
List job interim results

Return the interim results from this job. Interim results are kept two days after the job has finished running.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | A job ID | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]

### Return type

**String**

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/plain, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_details_jid

> models::JobResponse get_job_details_jid(id, ibm_api_version, exclude_params)
List job details

List the details about the specified quantum program job.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Identifier of an existing job | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]
**exclude_params** | Option<**bool**> | Exclude job params from the response |  |[default to false]

### Return type

[**models::JobResponse**](JobResponse.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_metrics_jid

> models::JobMetrics get_job_metrics_jid(id, ibm_api_version)
Get job metrics

Gets metrics of specified job

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | A job ID | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]

### Return type

[**models::JobMetrics**](JobMetrics.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_results_jid

> String get_job_results_jid(id, ibm_api_version)
List job results

Return the final result from this job.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | A job ID | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]

### Return type

**String**

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/plain, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_jog_logs_jid

> String get_jog_logs_jid(id, ibm_api_version)
List job logs

List all job logs for the specified job.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | A job ID | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]

### Return type

**String**

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/plain, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_transpiled_circuits_jid

> models::JobsTranspiledCircuitsResponse get_transpiled_circuits_jid(id, ibm_api_version)
Get job transpiled circuits

Return a presigned download URL for the transpiled circuits. Currently supported only for sampler primitive.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | A job ID | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]

### Return type

[**models::JobsTranspiledCircuitsResponse**](JobsTranspiledCircuitsResponse.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_jobs

> models::JobsResponse list_jobs(ibm_api_version, limit, offset, pending, program, backend, created_after, created_before, sort, tags, session_id, exclude_params)
List jobs

List the quantum program jobs you have run.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]
**limit** | Option<**i32**> | Number of results to return at a time |  |[default to 200]
**offset** | Option<**i32**> | Number of results to offset when retrieving the list of jobs |  |
**pending** | Option<**bool**> | Returns 'Queued' and 'Running' jobs if true.  Returns 'Completed', 'Cancelled', and 'Failed' jobs if false. |  |
**program** | Option<**String**> | Program ID to filter jobs |  |
**backend** | Option<**String**> | Backend to filter jobs |  |
**created_after** | Option<**String**> | Job created after filter |  |
**created_before** | Option<**String**> | Job created before filter |  |
**sort** | Option<**String**> | Sort jobs by created time ASC or DESC (default) |  |
**tags** | Option<[**Vec<String>**](String.md)> | Tags to filter jobs |  |
**session_id** | Option<**String**> | Session ID to filter jobs |  |
**exclude_params** | Option<**bool**> | Exclude job params from the response |  |[default to true]

### Return type

[**models::JobsResponse**](JobsResponse.md)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## replace_job_tags

> replace_job_tags(id, ibm_api_version, replace_job_tags_request)
Replace job tags

Replace job tags

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | A job ID | [required] |
**ibm_api_version** | Option<**String**> |  |  |[default to 2025-01-01]
**replace_job_tags_request** | Option<[**ReplaceJobTagsRequest**](ReplaceJobTagsRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[IBMCloudAuth](../README.md#IBMCloudAuth), [IBMCloudAPIKey](../README.md#IBMCloudAPIKey), [Backend-Authentication](../README.md#Backend-Authentication), [external-service-token](../README.md#external-service-token), [ServiceCRN](../README.md#ServiceCRN)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

