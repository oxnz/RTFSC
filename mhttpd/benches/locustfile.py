from locust import FastHttpUser, User, task
import httpx

# class SimpleUser(FastHttpUser):

#     host = 'http://127.0.0.1:8000'

#     @task(10)
#     def get(self):
#         self.client.get('/index.html')

class H2CUser(User):

    host = 'http://127.0.0.1:8000'

    def __init__(self, environment):
        super().__init__(environment)
        self.client = httpx.Client(http1=False, http2=True)
        self.request_event = environment.events.request

    @task
    def get(self):
        response = self.client.get(self.host)
        request_meta = {
            "request_type": "http2",
            "name": 'h2c',
            "response_length": len(response.content),
            "response": response.text,
            "context": {},  # see HttpUser if you actually want to implement contexts
            "exception": None,
            'response_time': response.elapsed.total_seconds() * 1000,
        }
        try:
            response.raise_for_status()
        except Exception as e:
            response['exception'] = e
        self.request_event.fire(**request_meta)
