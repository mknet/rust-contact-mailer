#!/usr/bin/env bash
ansible-playbook -i ansible/hosts ansible/playbook.yml --vault-password-file ansible/vault-env