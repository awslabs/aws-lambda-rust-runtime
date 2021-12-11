def handler(event, context):
    return {
        "message": event["command"].upper()
    }