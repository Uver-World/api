## Ansible PlayBook

#### In order to run the ansible playbook, make sure you meet the following requirements:

- Ansible is installed on your system
  > You can check it with: `ansible --version`

- The environment file is located in the same folder

- You have configured the server's host
  > Run the following to check if your host configuration is fine: `ansible-playbook -i hosts playbook.yml --list-hosts`

#### Now you can run the playbook from the root folder:

- `ansible-playbook -i ansible/hosts ansible/playbook.yml -f 10 -k`
