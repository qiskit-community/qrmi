/*
 * This code is part of Qiskit.
 *
 * Copyright (C) 2025 IBM, UKRI-STFC (Hartree Centre)
 *
 * This code is licensed under the Apache License, Version 2.0. You may
 * obtain a copy of this license in the LICENSE.txt file in the root directory
 * of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
 *
 * Any modifications or derivative works of this code must retain this
 * copyright notice, and modified files need to carry a notice indicating
 * that they have been altered from the originals.
 */
#include <stdint.h>
#include <stdio.h>
#include <unistd.h>

#include "qrmi.h"

extern void load_dotenv();
extern const char *read_file(const char *);

int main(int argc, char *argv[]) {

  if (argc != 4) {
    fprintf(stderr, "qiskit_runtime_service <backend_name> <primitive input "
                    "file> <program id>\n");
    return EXIT_SUCCESS;
  }

  load_dotenv();

  QrmiQuantumResource *qrmi =
      qrmi_resource_new(argv[1], QRMI_RESOURCE_TYPE_QISKIT_RUNTIME_SERVICE);
  if (!qrmi) {
    fprintf(stderr, "Failed to create QRMI for %s.\n", argv[1]);
    return EXIT_FAILURE;
  }

  bool is_accessible = false;
  QrmiReturnCode rc = qrmi_resource_is_accessible(qrmi, &is_accessible);
  if (rc == QRMI_RETURN_CODE_SUCCESS) {
    if (is_accessible == false) {
      fprintf(stderr, "%s cannot be accessed.\n", argv[1]);
      goto error;
    }
  } else {
    fprintf(stderr, "qrmi_resource_is_accessible() failed.\n");
    goto error;
  }

  char *acquisition_token = NULL;
  rc = qrmi_resource_acquire(qrmi, &acquisition_token);
  if (rc != QRMI_RETURN_CODE_SUCCESS) {
    fprintf(stderr, "qrmi_resource_acquire() failed.\n");
    goto error;
  }
  fprintf(stdout, "acquisition_token = %s\n", acquisition_token);

  char *target = NULL;
  rc = qrmi_resource_target(qrmi, &target);
  if (rc == QRMI_RETURN_CODE_SUCCESS) {
    fprintf(stdout, "target = %s\n", target);
    qrmi_string_free((char *)target);
  } else {
    fprintf(stderr, "qrmi_resource_target() failed.\n");
    goto error;
  }

  const char *input = read_file(argv[2]);

  QrmiPayload payload;
  payload.tag = QRMI_PAYLOAD_QISKIT_PRIMITIVE;
  payload.QISKIT_PRIMITIVE.input = (char *)input;
  payload.QISKIT_PRIMITIVE.program_id = argv[3];

  char *job_id = NULL;
  rc = qrmi_resource_task_start(qrmi, &payload, &job_id);
  if (rc != QRMI_RETURN_CODE_SUCCESS) {
    fprintf(stderr, "failed to start a task.\n");
    free((void *)input);
    goto error;
  }
  fprintf(stdout, "Job ID: %s\n", job_id);
  free((void *)input);

  QrmiTaskStatus status;
  while (1) {
    rc = qrmi_resource_task_status(qrmi, job_id, &status);
    fprintf(stdout, "rc = %d, status = %d\n", rc, status);
    if (rc != QRMI_RETURN_CODE_SUCCESS || (status != QRMI_TASK_STATUS_RUNNING &&
                                           status != QRMI_TASK_STATUS_QUEUED)) {
      break;
    }
    sleep(1);
  }

  rc = qrmi_resource_task_status(qrmi, job_id, &status);
  if (rc == QRMI_RETURN_CODE_SUCCESS && status == QRMI_TASK_STATUS_COMPLETED) {
    char *result = NULL;
    rc = qrmi_resource_task_result(qrmi, job_id, &result);
    if (rc == QRMI_RETURN_CODE_SUCCESS) {
      fprintf(stdout, "%s\n", result);
      qrmi_string_free((char *)result);
    } else {
      fprintf(stderr, "qrmi_resource_task_result() failed.\n");
    }
  } else if (status == QRMI_TASK_STATUS_FAILED) {
    fprintf(stderr, "Failed.\n");
  } else if (status == QRMI_TASK_STATUS_CANCELLED) {
    fprintf(stderr, "Cancelled.\n");
  }

  qrmi_resource_task_stop(qrmi, job_id);

  qrmi_string_free((char *)job_id);

  rc = qrmi_resource_release(qrmi, acquisition_token);
  fprintf(stdout, "qrmi_resource_release rc = %d\n", rc);
  qrmi_string_free((char *)acquisition_token);

  qrmi_resource_free(qrmi);

  return EXIT_SUCCESS;

error:
  qrmi_resource_free(qrmi);
  return EXIT_FAILURE;
}
