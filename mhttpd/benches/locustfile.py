from locust import FastHttpUser, task

class SimpleUser(FastHttpUser):

    host = 'http://127.0.0.1:8000'

    @task(10)
    def get(self):
        self.client.get('/index.html')