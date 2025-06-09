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
    fprintf(stderr, "qiskit_runtime_service <backend_name> <primitive input file> <program id>\n");
    return 0;
  }

  load_dotenv();

  IBMQiskitRuntimeService *qrmi = qrmi_ibmqrs_new(argv[1]);
  bool is_accessible = false;
  int rc = qrmi_ibmqrs_is_accessible(qrmi, &is_accessible);
  if (rc == QRMI_SUCCESS) {
    if (is_accessible == false) {
      fprintf(stderr, "%s cannot be accessed.\n", argv[1]);
      return -1;
    }
  } else {
    fprintf(stderr, "qrmi_ibmqrs_is_accessible() failed.\n");
    return -1;
  }

  const char *acquisition_token = qrmi_ibmqrs_acquire(qrmi);
  fprintf(stdout, "acquisition_token = %s\n", acquisition_token);

  const char *target = qrmi_ibmqrs_target(qrmi);
  fprintf(stdout, "target = %s\n", target);
  qrmi_free_string((char *)target);

  const char *input = read_file(argv[2]);
  const char *job_id = qrmi_ibmqrs_task_start(qrmi, argv[3], input);
  if (job_id == NULL) {
    fprintf(stderr, "failed to start a task.\n");
    free((void*)input);
    return -1;
  }
  fprintf(stdout, "Job ID: %s\n", job_id);
  free((void*)input);

  TaskStatus status;
  while (1) {
    rc = qrmi_ibmqrs_task_status(qrmi, job_id, &status);
    fprintf(stdout, "rc = %d, status = %d\n", rc, status);
    if (rc != QRMI_SUCCESS || (status != RUNNING && status != QUEUED)) {
      break;
    }
    sleep(1);
  }

  rc = qrmi_ibmqrs_task_status(qrmi, job_id, &status);
  if (rc == QRMI_SUCCESS && status == COMPLETED) {
    const char *result = qrmi_ibmqrs_task_result(qrmi, job_id);
    fprintf(stdout, "%s\n", result);
    qrmi_free_string((char *)result);
  }
  else if (status == FAILED) {
    fprintf(stderr, "Failed.\n");
  }
  else if (status == CANCELLED) {
    fprintf(stderr, "Cancelled.\n");
  }

  qrmi_ibmqrs_task_stop(qrmi, job_id);

  qrmi_free_string((char *)job_id);

  rc = qrmi_ibmqrs_release(qrmi, acquisition_token);
  fprintf(stdout, "qrmi_ibmqrs_release rc = %d\n", rc);
  qrmi_free_string((char *)acquisition_token);

  qrmi_ibmqrs_free(qrmi);

  return 0;
}
